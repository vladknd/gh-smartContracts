use ic_cdk::{init, query, update, post_upgrade};
use candid::Principal;

mod types;
mod state;
mod constants;
mod service;

use types::*;
use state::*;
use service::*;

// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
    if let Some(gov_id) = args.governance_canister_id {
        GOVERNANCE_CANISTER_ID.with(|id| id.borrow_mut().set(gov_id).expect("Failed to set Governance Canister ID"));
    }
}

#[post_upgrade]
fn post_upgrade() {
    // Rebuild quiz index to ensure hashes are stable/up-to-date
    rebuild_quiz_index();

    // Schedule job resumption after a short delay (can't spawn directly in post_upgrade)
    ic_cdk_timers::set_timer(std::time::Duration::from_secs(1), || {
        ic_cdk::spawn(async {
            resume_incomplete_jobs().await;
        });
    });
}

// ============================================================================
// CONTENT NODE MANAGEMENT
// ============================================================================

/// Add or update a content node (public function - admin only)
#[update]
fn add_content_node(node: ContentNode) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let gov_id = GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get());
    
    // Check if authorized (governance or controller)
    let is_gov = gov_id != Principal::anonymous() && caller == gov_id;
    if !is_gov && !ic_cdk::api::is_controller(&caller) {
         return Err("Unauthorized".to_string());
    }
    
    add_content_node_internal(node, None)
}

/// Add multiple content nodes (batch operation)
#[update]
fn add_content_nodes(nodes: Vec<ContentNode>) -> Result<u32, String> {
    let caller = ic_cdk::caller();
    let gov_id = GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get());
    
    // Check if authorized (governance or controller)
    let is_gov = gov_id != Principal::anonymous() && caller == gov_id;
    if !is_gov && !ic_cdk::api::is_controller(&caller) {
         return Err("Unauthorized".to_string());
    }

    let mut added = 0;
    for node in nodes {
        add_content_node_internal(node, None)?;
        added += 1;
    }
    Ok(added)
}

/// Get a content node by ID (public version without answers)
#[query]
fn get_content_node(id: String) -> Option<PublicContentNode> {
    CONTENT_NODES.with(|c| {
        c.borrow().get(&id).map(|node| to_public_node(&node))
    })
}

/// Get children of a content node
#[query]
fn get_children(parent_id: String) -> Vec<PublicContentNode> {
    let child_ids = CHILDREN_INDEX.with(|idx| {
        idx.borrow().get(&parent_id).map(|c| c.0.clone()).unwrap_or_default()
    });
    
    CONTENT_NODES.with(|c| {
        let nodes = c.borrow();
        let mut children: Vec<PublicContentNode> = child_ids.iter()
            .filter_map(|id| nodes.get(id).map(|n| to_public_node(&n)))
            .collect();
        children.sort_by_key(|n| n.order);
        children
    })
}

/// Get all root nodes (nodes without parents)
#[query]
fn get_root_nodes() -> Vec<PublicContentNode> {
    CONTENT_NODES.with(|c| {
        let mut roots: Vec<PublicContentNode> = c.borrow()
            .iter()
            .filter(|(_, node)| node.parent_id.is_none())
            .map(|(_, node)| to_public_node(&node))
            .collect();
        roots.sort_by_key(|n| n.order);
        roots
    })
}

/// Delete a content node
#[update]
async fn delete_content_node(id: String, proposal_id: u64) -> Result<(), String> {
    // Only governance or admin
    let gov_id = GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get());
    if gov_id != Principal::anonymous() {
        if ic_cdk::caller() != gov_id && !ic_cdk::api::is_controller(&ic_cdk::caller()) {
             return Err("Unauthorized".to_string());
        }
    } else if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized".to_string());
    }
    delete_content_node_impl(id, proposal_id)
}

// ============================================================================
// QUIZ FUNCTIONS
// ============================================================================

/// Get quiz cache data for user profile shards
#[query]
fn get_quiz_data(content_id: String) -> Option<QuizCacheData> {
    QUIZ_INDEX.with(|q| q.borrow().get(&content_id))
}

/// Get all quiz cache data (for full shard sync)
#[query]
fn get_all_quiz_cache_data() -> Vec<(String, QuizCacheData)> {
    QUIZ_INDEX.with(|q| {
        q.borrow().iter().collect()
    })
}

/// Verify quiz answers (called by user_profile shards or directly)
#[update]
fn verify_quiz(content_id: String, answers: Vec<u8>) -> (bool, u64, u64) {
    verify_quiz_impl(content_id, answers)
}

// ============================================================================
// CONTENT LOADING (RESILIENT)
// ============================================================================

/// Start loading content from staging (called by governance on proposal approval)
#[update]
async fn start_content_load(
    proposal_id: u64,
    staging_canister: Principal,
    staging_path: String,
    content_hash: String,
    total_units: u32,
) -> Result<(), String> {
    // Auth check
    let caller = ic_cdk::caller();
    let gov_id = GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get());
    
    // Check if authorized (governance or controller)
    let is_gov = gov_id != Principal::anonymous() && caller == gov_id;
    if !is_gov && !ic_cdk::api::is_controller(&caller) {
         return Err("Unauthorized".to_string());
    }
    
    start_content_load_impl(proposal_id, staging_canister, staging_path, content_hash, total_units).await
}

/// Continue loading (self-call pattern for resilience)
#[update]
async fn continue_loading(proposal_id: u64) -> Result<(), String> {
    // Allow self-calls or controllers
    let caller = ic_cdk::caller();
    if caller != ic_cdk::api::id() && !ic_cdk::api::is_controller(&caller) {
        return Err("Unauthorized".to_string());
    }
    
    continue_loading_impl(proposal_id).await
}

/// Resume loading after error or pause
#[update]
async fn resume_loading(proposal_id: u64) -> Result<(), String> {
    // Auth check
    let caller = ic_cdk::caller();
    let gov_id = GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get());
    
    // Check if authorized (governance or controller)
    let is_gov = gov_id != Principal::anonymous() && caller == gov_id;
    if !is_gov && !ic_cdk::api::is_controller(&caller) {
         return Err("Unauthorized".to_string());
    }

    let job = LOADING_JOBS.with(|jobs| jobs.borrow().get(&proposal_id))
        .ok_or("Loading job not found")?;
        
    if job.status == LoadingStatus::Completed {
        return Err("Job already completed".to_string());
    }
    
    // Update status to InProgress
    LOADING_JOBS.with(|jobs| {
        jobs.borrow_mut().insert(proposal_id, {
            let mut j = job.clone();
            j.status = LoadingStatus::InProgress;
            j.last_error = None;
            j.updated_at = ic_cdk::api::time();
            j
        });
    });
    
    // Kick off
    continue_loading_impl(proposal_id).await
}

/// Get loading status for a proposal
#[query]
fn get_loading_status(proposal_id: u64) -> Option<LoadingJob> {
    LOADING_JOBS.with(|jobs| jobs.borrow().get(&proposal_id))
}

/// Get all loading jobs
#[query]
fn get_all_loading_jobs() -> Vec<LoadingJob> {
    LOADING_JOBS.with(|jobs| {
        jobs.borrow().iter().map(|(_, job)| job).collect()
    })
}

// ============================================================================
// VERSION HISTORY QUERIES
// ============================================================================

/// Get version history for a content node
#[query]
fn get_content_version_history(content_id: String) -> Vec<(u64, ContentSnapshot)> {
    VERSION_HISTORY.with(|h| {
        h.borrow()
            .iter()
            .filter(|(key, _)| key.content_id == content_id)
            .map(|(key, snapshot)| (key.version, snapshot))
            .collect()
    })
}

/// Get content at a specific version
#[query]
fn get_content_at_version(content_id: String, version: u64) -> Option<ContentNode> {
    VERSION_HISTORY.with(|h| {
        h.borrow()
            .get(&VersionKey { content_id, version })
            .map(|snapshot| snapshot.content)
    })
}

/// Get current version number for a content node
#[query]
fn get_content_current_version(content_id: String) -> u64 {
    get_content_version(&content_id)
}

/// Get all changes made by a specific proposal
#[query]
fn get_changes_by_proposal(proposal_id: u64) -> Vec<(String, ChangeType)> {
    VERSION_HISTORY.with(|h| {
        h.borrow()
            .iter()
            .filter(|(_, snapshot)| snapshot.modified_by_proposal == proposal_id)
            .map(|(key, snapshot)| (key.content_id, snapshot.change_type))
            .collect()
    })
}

// ============================================================================
// GLOBAL VERSION
// ============================================================================

/// Get global content version
#[query]
fn get_content_version_global() -> u64 {
    CONTENT_VERSION.with(|v| *v.borrow().get())
}

// ============================================================================
// STATISTICS
// ============================================================================

/// Get content statistics: (node_count, quiz_count)
#[query]
fn get_content_stats() -> (u64, u64) {
    let node_count = CONTENT_NODES.with(|c| c.borrow().len());
    let quiz_count = QUIZ_INDEX.with(|q| q.borrow().len());
    
    (node_count, quiz_count)
}

ic_cdk::export_candid!();
