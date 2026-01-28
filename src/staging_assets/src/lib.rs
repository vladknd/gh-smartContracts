//! # Staging Assets Canister
//! 
//! This canister provides temporary storage for content metadata before
//! governance approval. Content is staged here, a proposal is created,
//! and if approved, the learning_engine fetches content from here.

mod types;
mod state;
mod service;

use ic_cdk::{init, query, update};
use candid::Principal;
use crate::types::*;
use crate::state::*;

// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init(args: InitArgs) {
    GOVERNANCE_CANISTER_ID.with(|id| {
        id.borrow_mut().set(args.governance_canister_id)
            .expect("Failed to set governance canister ID");
    });
    LEARNING_ENGINE_ID.with(|id| {
        id.borrow_mut().set(args.learning_engine_id)
            .expect("Failed to set learning engine ID");
    });
}

// ============================================================================
// STAGING FUNCTIONS
// ============================================================================

/// Stage content for governance approval
/// Returns the content_hash which is used to reference this staged content
#[update]
fn stage_content(
    title: String,
    description: String,
    nodes: Vec<ContentNode>,
) -> Result<String, String> {
    service::stage_content_impl(ic_cdk::caller(), title, description, nodes)
}

/// Associate a proposal with staged content
#[update]
fn set_proposal_id(content_hash: String, proposal_id: u64) -> Result<(), String> {
    service::set_proposal_id_impl(ic_cdk::caller(), content_hash, proposal_id)
}

/// Mark content as loading (called by governance when proposal approved)
#[update]
fn mark_loading(content_hash: String) -> Result<(), String> {
    service::mark_loading_impl(ic_cdk::caller(), content_hash)
}

/// Mark content as loaded (called by learning engine after successful load)
#[update]
fn mark_loaded(content_hash: String) -> Result<(), String> {
    service::mark_loaded_impl(ic_cdk::caller(), content_hash)
}

/// Mark content as rejected (called by governance when proposal rejected)
#[update]
fn mark_rejected(content_hash: String) -> Result<(), String> {
    service::mark_rejected_impl(ic_cdk::caller(), content_hash)
}

/// Delete staged content (cleanup after loading or rejection)
#[update]
fn delete_staged_content(content_hash: String) -> Result<(), String> {
    service::delete_staged_content_impl(ic_cdk::caller(), content_hash)
}

// ============================================================================
// CONTENT RETRIEVAL FUNCTIONS (Called by Learning Engine)
// ============================================================================

/// Get a chunk of content nodes (called by learning_engine during loading)
#[query]
fn get_content_chunk(content_hash: String, offset: u32, limit: u32) -> Vec<ContentNode> {
    let caller = ic_cdk::caller();
    
    // Only learning engine, governance, or controller can read chunks
    if !service::is_learning_engine(&caller) 
        && !service::is_governance_canister(&caller) 
        && !ic_cdk::api::is_controller(&caller) {
        return Vec::new();
    }
    
    STAGED_CONTENT.with(|s| {
        if let Some(content) = s.borrow().get(&content_hash) {
            content.nodes
                .iter()
                .skip(offset as usize)
                .take(limit as usize)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    })
}

/// Get all content nodes (for small content packages)
#[query]
fn get_all_content_nodes(content_hash: String) -> Result<Vec<ContentNode>, String> {
    let caller = ic_cdk::caller();
    
    // Only learning engine, governance, or controller can read
    if !service::is_learning_engine(&caller) 
        && !service::is_governance_canister(&caller) 
        && !ic_cdk::api::is_controller(&caller) {
        return Err("Not authorized".to_string());
    }
    
    STAGED_CONTENT.with(|s| {
        s.borrow()
            .get(&content_hash)
            .map(|c| c.nodes.clone())
            .ok_or("Staged content not found".to_string())
    })
}

// ============================================================================
// QUERY FUNCTIONS
// ============================================================================

/// Get staged content metadata (without the actual nodes)
#[query]
fn get_staged_content_info(content_hash: String) -> Option<StagedContentInfo> {
    STAGED_CONTENT.with(|s| {
        s.borrow().get(&content_hash).map(|c| StagedContentInfo {
            content_hash: c.content_hash,
            title: c.title,
            description: c.description,
            node_count: c.node_count,
            stager: c.stager,
            staged_at: c.staged_at,
            proposal_id: c.proposal_id,
            status: c.status,
        })
    })
}

/// Check if staged content exists
#[query]
fn staged_content_exists(content_hash: String) -> bool {
    STAGED_CONTENT.with(|s| s.borrow().contains_key(&content_hash))
}

/// List all staged content (metadata only)
#[query]
fn list_staged_content() -> Vec<StagedContentInfo> {
    STAGED_CONTENT.with(|s| {
        s.borrow()
            .iter()
            .map(|(_, c)| StagedContentInfo {
                content_hash: c.content_hash,
                title: c.title,
                description: c.description,
                node_count: c.node_count,
                stager: c.stager,
                staged_at: c.staged_at,
                proposal_id: c.proposal_id,
                status: c.status,
            })
            .collect()
    })
}

/// Get staged content by stager
#[query]
fn get_staged_by_stager(stager: Principal) -> Vec<StagedContentInfo> {
    STAGED_CONTENT.with(|s| {
        s.borrow()
            .iter()
            .filter(|(_, c)| c.stager == stager)
            .map(|(_, c)| StagedContentInfo {
                content_hash: c.content_hash.clone(),
                title: c.title.clone(),
                description: c.description.clone(),
                node_count: c.node_count,
                stager: c.stager,
                staged_at: c.staged_at,
                proposal_id: c.proposal_id,
                status: c.status.clone(),
            })
            .collect()
    })
}

/// Get count of staged content
#[query]
fn get_staged_count() -> u64 {
    STAGED_CONTENT.with(|s| s.borrow().len())
}

// ============================================================================
// ADMIN FUNCTIONS
// ============================================================================

/// Add an allowed stager (controller only)
#[update]
fn add_allowed_stager(principal: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Only controllers can add stagers".to_string());
    }
    
    ALLOWED_STAGERS.with(|u| {
        let mut list = u.borrow().get().clone();
        if !list.0.contains(&principal) {
            list.0.push(principal);
            u.borrow_mut().set(list).expect("Failed to update stagers");
        }
    });
    
    Ok(())
}

/// Remove an allowed stager (controller only)
#[update]
fn remove_allowed_stager(principal: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Only controllers can remove stagers".to_string());
    }
    
    ALLOWED_STAGERS.with(|u| {
        let mut list = u.borrow().get().clone();
        list.0.retain(|p| p != &principal);
        u.borrow_mut().set(list).expect("Failed to update stagers");
    });
    
    Ok(())
}

/// Get list of allowed stagers
#[query]
fn get_allowed_stagers() -> Vec<Principal> {
    ALLOWED_STAGERS.with(|u| u.borrow().get().0.clone())
}

/// Update governance canister ID (controller only)
#[update]
fn set_governance_canister_id(principal: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Only controllers can update governance ID".to_string());
    }
    
    GOVERNANCE_CANISTER_ID.with(|id| {
        id.borrow_mut().set(principal).expect("Failed to set governance ID");
    });
    
    Ok(())
}

/// Update learning engine ID (controller only)
#[update]
fn set_learning_engine_id(principal: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Only controllers can update learning engine ID".to_string());
    }
    
    LEARNING_ENGINE_ID.with(|id| {
        id.borrow_mut().set(principal).expect("Failed to set learning engine ID");
    });
    
    Ok(())
}

/// Get governance canister ID
#[query]
fn get_governance_canister_id() -> Principal {
    GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get())
}

/// Get learning engine ID
#[query]
fn get_learning_engine_id() -> Principal {
    LEARNING_ENGINE_ID.with(|id| *id.borrow().get())
}

ic_cdk::export_candid!();
