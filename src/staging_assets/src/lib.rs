use std::cell::RefCell;
use std::borrow::Cow;
use ic_cdk::{init, query, update};
use candid::{CandidType, Deserialize, Principal};
use candid::{Encode, Decode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use sha2::{Sha256, Digest};

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// STAGING ASSETS CANISTER
// ============================================================================
// 
// This canister provides temporary storage for content metadata before
// governance approval. Content is staged here, a proposal is created,
// and if approved, the learning_engine fetches content from here.
// 
// After successful loading, content is deleted from staging.
// 
// Key features:
// - Store ContentNode arrays as staged content
// - Provide chunks to learning_engine during loading
// - Automatic cleanup after successful load
// - Only authorized principals can stage content

// ============================================================================
// DATA STRUCTURES - Match Learning Engine Types
// ============================================================================

/// Media type for content attachments
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    PDF,
}

/// Media content attached to a node
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MediaContent {
    pub media_type: MediaType,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<u32>,
    pub file_hash: Option<String>,
}

/// A quiz question with answer
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QuizQuestion {
    pub question: String,
    pub options: Vec<String>,
    pub answer: u8,
}

/// Quiz data attached to a content node
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QuizData {
    pub questions: Vec<QuizQuestion>,
}

/// The universal content node
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ContentNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub order: u32,
    pub display_type: String,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub paraphrase: Option<String>,
    pub media: Option<MediaContent>,
    pub quiz: Option<QuizData>,
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u64,
}

// ============================================================================
// STAGING-SPECIFIC STRUCTURES
// ============================================================================

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
struct InitArgs {
    /// Governance canister that can read and delete staged content
    governance_canister_id: Principal,
    /// Learning engine that can read staged content
    learning_engine_id: Principal,
}

/// Staged content package
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StagedContent {
    /// SHA256 hash of the serialized content (content ID)
    pub content_hash: String,
    /// Human-readable title
    pub title: String,
    /// Description of what this content package contains
    pub description: String,
    /// The actual content nodes
    pub nodes: Vec<ContentNode>,
    /// Number of nodes
    pub node_count: u32,
    /// Who staged this content
    pub stager: Principal,
    /// When it was staged
    pub staged_at: u64,
    /// Associated proposal ID (set when proposal is created)
    pub proposal_id: Option<u64>,
    /// Status
    pub status: StagingStatus,
}

impl Storable for StagedContent {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode StagedContent")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 50_000_000, // 50MB max per staged content
        is_fixed_size: false,
    };
}

/// Status of staged content
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum StagingStatus {
    /// Waiting for proposal to be created
    Pending,
    /// Proposal created, waiting for vote
    ProposalCreated,
    /// Currently being loaded to learning engine
    Loading,
    /// Successfully loaded (can be deleted)
    Loaded,
    /// Proposal rejected (can be deleted or resubmitted)
    Rejected,
}

/// List wrapper for stable storage
#[derive(CandidType, Deserialize, Clone, Debug, Default)]
struct PrincipalList(Vec<Principal>);

impl Storable for PrincipalList {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode PrincipalList")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 10000,
        is_fixed_size: false,
    };
}

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    /// Staged content: content_hash -> StagedContent
    static STAGED_CONTENT: RefCell<StableBTreeMap<String, StagedContent, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    /// Governance canister ID
    static GOVERNANCE_CANISTER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Learning engine ID
    static LEARNING_ENGINE_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Allowed stagers (empty = anyone with sufficient privileges)
    static ALLOWED_STAGERS: RefCell<StableCell<PrincipalList, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
            PrincipalList::default()
        ).unwrap()
    );
}

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
// HELPER FUNCTIONS
// ============================================================================

/// Check if caller is allowed to stage content
fn is_allowed_stager(caller: &Principal) -> bool {
    // Controllers always allowed
    if ic_cdk::api::is_controller(caller) {
        return true;
    }
    
    ALLOWED_STAGERS.with(|u| {
        let list = u.borrow().get().clone();
        // Empty list = anyone can stage
        list.0.is_empty() || list.0.contains(caller)
    })
}

/// Check if caller is the governance canister
fn is_governance_canister(caller: &Principal) -> bool {
    GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get() == *caller)
}

/// Check if caller is the learning engine
fn is_learning_engine(caller: &Principal) -> bool {
    LEARNING_ENGINE_ID.with(|id| *id.borrow().get() == *caller)
}

/// Compute SHA256 hash of data
fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Encode bytes to hex
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
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
    let caller = ic_cdk::caller();
    
    if !is_allowed_stager(&caller) {
        return Err("Not authorized to stage content".to_string());
    }
    
    if nodes.is_empty() {
        return Err("Cannot stage empty content".to_string());
    }
    
    if title.is_empty() || title.len() > 200 {
        return Err("Title must be 1-200 characters".to_string());
    }
    
    // Compute hash of the content
    let content_bytes = Encode!(&nodes).map_err(|e| format!("Failed to encode: {}", e))?;
    let content_hash = compute_hash(&content_bytes);
    
    // Check if already staged
    if STAGED_CONTENT.with(|s| s.borrow().contains_key(&content_hash)) {
        return Err("Content with this hash already staged".to_string());
    }
    
    let now = ic_cdk::api::time();
    let node_count = nodes.len() as u32;
    
    let staged = StagedContent {
        content_hash: content_hash.clone(),
        title,
        description,
        nodes,
        node_count,
        stager: caller,
        staged_at: now,
        proposal_id: None,
        status: StagingStatus::Pending,
    };
    
    STAGED_CONTENT.with(|s| s.borrow_mut().insert(content_hash.clone(), staged));
    
    Ok(content_hash)
}

/// Associate a proposal with staged content
#[update]
fn set_proposal_id(content_hash: String, proposal_id: u64) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Only the original stager, governance canister, or controller can do this
    let mut content = STAGED_CONTENT.with(|s| s.borrow().get(&content_hash))
        .ok_or("Staged content not found")?;
    
    if content.stager != caller 
        && !is_governance_canister(&caller) 
        && !ic_cdk::api::is_controller(&caller) {
        return Err("Not authorized".to_string());
    }
    
    content.proposal_id = Some(proposal_id);
    content.status = StagingStatus::ProposalCreated;
    
    STAGED_CONTENT.with(|s| s.borrow_mut().insert(content_hash, content));
    
    Ok(())
}

/// Mark content as loading (called by governance when proposal approved)
#[update]
fn mark_loading(content_hash: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    if !is_governance_canister(&caller) && !ic_cdk::api::is_controller(&caller) {
        return Err("Only governance canister can mark as loading".to_string());
    }
    
    let mut content = STAGED_CONTENT.with(|s| s.borrow().get(&content_hash))
        .ok_or("Staged content not found")?;
    
    content.status = StagingStatus::Loading;
    
    STAGED_CONTENT.with(|s| s.borrow_mut().insert(content_hash, content));
    
    Ok(())
}

/// Mark content as loaded (called by learning engine after successful load)
#[update]
fn mark_loaded(content_hash: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    if !is_learning_engine(&caller) 
        && !is_governance_canister(&caller) 
        && !ic_cdk::api::is_controller(&caller) {
        return Err("Only learning engine or governance can mark as loaded".to_string());
    }
    
    let mut content = STAGED_CONTENT.with(|s| s.borrow().get(&content_hash))
        .ok_or("Staged content not found")?;
    
    content.status = StagingStatus::Loaded;
    
    STAGED_CONTENT.with(|s| s.borrow_mut().insert(content_hash, content));
    
    Ok(())
}

/// Mark content as rejected (called by governance when proposal rejected)
#[update]
fn mark_rejected(content_hash: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    if !is_governance_canister(&caller) && !ic_cdk::api::is_controller(&caller) {
        return Err("Only governance canister can mark as rejected".to_string());
    }
    
    let mut content = STAGED_CONTENT.with(|s| s.borrow().get(&content_hash))
        .ok_or("Staged content not found")?;
    
    content.status = StagingStatus::Rejected;
    
    STAGED_CONTENT.with(|s| s.borrow_mut().insert(content_hash, content));
    
    Ok(())
}

/// Delete staged content (cleanup after loading or rejection)
#[update]
fn delete_staged_content(content_hash: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    let content = STAGED_CONTENT.with(|s| s.borrow().get(&content_hash))
        .ok_or("Staged content not found")?;
    
    // Can delete if: original stager, governance, learning engine, or controller
    // But only if status allows
    if content.stager != caller 
        && !is_governance_canister(&caller) 
        && !is_learning_engine(&caller)
        && !ic_cdk::api::is_controller(&caller) {
        return Err("Not authorized to delete".to_string());
    }
    
    // Only allow deletion in certain states
    match content.status {
        StagingStatus::Loaded | StagingStatus::Rejected => {
            STAGED_CONTENT.with(|s| s.borrow_mut().remove(&content_hash));
            Ok(())
        }
        StagingStatus::Pending => {
            // Original stager can delete pending content
            if content.stager == caller || ic_cdk::api::is_controller(&caller) {
                STAGED_CONTENT.with(|s| s.borrow_mut().remove(&content_hash));
                Ok(())
            } else {
                Err("Cannot delete pending content".to_string())
            }
        }
        _ => Err("Cannot delete content in current state".to_string()),
    }
}

// ============================================================================
// CONTENT RETRIEVAL FUNCTIONS (Called by Learning Engine)
// ============================================================================

/// Get a chunk of content nodes (called by learning_engine during loading)
#[query]
fn get_content_chunk(content_hash: String, offset: u32, limit: u32) -> Vec<ContentNode> {
    let caller = ic_cdk::caller();
    
    // Only learning engine, governance, or controller can read chunks
    if !is_learning_engine(&caller) 
        && !is_governance_canister(&caller) 
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
    if !is_learning_engine(&caller) 
        && !is_governance_canister(&caller) 
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

/// Staged content info (without nodes for efficiency)
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StagedContentInfo {
    pub content_hash: String,
    pub title: String,
    pub description: String,
    pub node_count: u32,
    pub stager: Principal,
    pub staged_at: u64,
    pub proposal_id: Option<u64>,
    pub status: StagingStatus,
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
