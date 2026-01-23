use std::cell::RefCell;
use std::borrow::Cow;
use ic_cdk::{init, query, update, post_upgrade};
use candid::{CandidType, Deserialize, Principal};
use candid::{Encode, Decode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// LEARNING ENGINE CANISTER
// ============================================================================
// 
// This canister stores educational content and quiz questions using a
// flexible tree-based content structure.
// 
// Key responsibilities:
// - Store content nodes (tree structure supporting any hierarchy depth)
// - Maintain quiz index for O(1) lookup
// - Provide quiz cache data to user profile shards
// - Support resilient content loading from staging
// - Track version history for audit trails

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
struct InitArgs {
    /// Principal ID of the staking hub (for future use)
    staking_hub_id: Principal,
    /// Principal ID of the governance canister
    governance_canister_id: Option<Principal>,
}

// ============================================================================
// MEDIA TYPES
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
    /// Type of media
    pub media_type: MediaType,
    /// URL to the media file (in asset canister or CDN)
    pub url: String,
    /// Optional thumbnail URL
    pub thumbnail_url: Option<String>,
    /// Duration in seconds (for video/audio)
    pub duration_seconds: Option<u32>,
    /// File hash for verification
    pub file_hash: Option<String>,
}

// ============================================================================
// QUIZ TYPES
// ============================================================================

/// A quiz question with answer
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QuizQuestion {
    /// The question text
    pub question: String,
    /// List of answer options
    pub options: Vec<String>,
    /// Index of the correct answer (0-based)
    pub answer: u8,
}

/// Quiz data attached to a content node
/// NOTE: No per-quiz config - all quizzes use GLOBAL_QUIZ_CONFIG
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QuizData {
    /// Quiz questions
    pub questions: Vec<QuizQuestion>,
}

/// Global quiz configuration - ONE config for ALL quizzes
/// This is the simplest and safest approach - any change requires governance proposal
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QuizConfig {
    /// Tokens awarded for completing ANY quiz
    pub reward_amount: u64,
    /// Pass threshold percentage (e.g., 60)
    pub pass_threshold_percent: u8,
    /// Maximum daily attempts per quiz per user (legacy, kept for compatibility)
    pub max_daily_attempts: u8,
    
    // =========================================================================
    // Quiz Limits - Maximum quizzes a user can complete in each time period
    // =========================================================================
    
    /// Maximum quizzes per day (default: 5)
    pub max_daily_quizzes: u8,
    /// Maximum quizzes per week (default: 25)
    pub max_weekly_quizzes: u8,
    /// Maximum quizzes per month (default: 70)
    pub max_monthly_quizzes: u8,
    /// Maximum quizzes per year (default: 600)
    pub max_yearly_quizzes: u16,
}

impl Default for QuizConfig {
    fn default() -> Self {
        Self {
            reward_amount: 100 * 100_000_000, // 100 tokens in e8s
            pass_threshold_percent: 60,
            max_daily_attempts: 5,
            // Quiz limits
            max_daily_quizzes: 5,
            max_weekly_quizzes: 25,
            max_monthly_quizzes: 70,
            max_yearly_quizzes: 600,
        }
    }
}

impl Storable for QuizConfig {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode QuizConfig")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,  // Increased for new fields
        is_fixed_size: false,
    };
}

/// Cached quiz config format for distribution to user_profile shards
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CachedQuizConfig {
    pub reward_amount: u64,
    pub pass_threshold_percent: u8,
    pub max_daily_attempts: u8,
    pub max_daily_quizzes: u8,
    pub max_weekly_quizzes: u8,
    pub max_monthly_quizzes: u8,
    pub max_yearly_quizzes: u16,
    pub version: u64,
}

/// Quiz cache data stored in QUIZ_INDEX for O(1) lookup by user profile shards
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QuizCacheData {
    /// Content ID this quiz belongs to
    pub content_id: String,
    /// Hash of each answer for local verification
    pub answer_hashes: Vec<[u8; 32]>,
    /// Number of questions
    pub question_count: u8,
    /// Version of this quiz data
    pub version: u64,
}

impl Storable for QuizCacheData {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode QuizCacheData")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1000,
        is_fixed_size: false,
    };
}

// ============================================================================
// CONTENT NODE - THE UNIVERSAL BUILDING BLOCK
// ============================================================================

/// The universal content node - can represent any content type
/// Nodes link together via parent_id to form a tree
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ContentNode {
    // Identity
    /// Unique ID like "book:1:ch:2:sec:3"
    pub id: String,
    
    // Tree Structure
    /// Points to parent (None = root node)
    pub parent_id: Option<String>,
    /// Order among siblings (1, 2, 3...)
    pub order: u32,
    
    // Display Info
    /// Display type: "Book", "Chapter", "Unit", "Lesson", "Module", etc.
    pub display_type: String,
    /// Title of this node
    pub title: String,
    /// Optional description
    pub description: Option<String>,
    
    // Content (optional - containers don't need it)
    /// Main text/markdown content
    pub content: Option<String>,
    /// Summary/paraphrase
    pub paraphrase: Option<String>,
    
    // Media (optional - just URLs, files stored in asset canister)
    pub media: Option<MediaContent>,
    
    // Quiz (optional - ANY node at ANY level can have a quiz)
    pub quiz: Option<QuizData>,
    
    // Metadata
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u64,
}

impl Default for ContentNode {
    fn default() -> Self {
        let now = ic_cdk::api::time();
        Self {
            id: String::new(),
            parent_id: None,
            order: 0,
            display_type: "Unit".to_string(),
            title: String::new(),
            description: None,
            content: None,
            paraphrase: None,
            media: None,
            quiz: None,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }
}

impl Storable for ContentNode {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ContentNode")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100000, // Generous limit for content
        is_fixed_size: false,
    };
}

/// Public version of ContentNode - EXCLUDES QUIZ ANSWERS
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PublicContentNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub order: u32,
    pub display_type: String,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub paraphrase: Option<String>,
    pub media: Option<MediaContent>,
    /// Quiz questions WITHOUT answers
    pub quiz: Option<PublicQuizData>,
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u64,
}

/// Public quiz data - EXCLUDES ANSWERS
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PublicQuizData {
    pub questions: Vec<PublicQuizQuestion>,
}

/// Public quiz question - EXCLUDES ANSWER
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PublicQuizQuestion {
    pub question: String,
    pub options: Vec<String>,
    // Note: answer field is intentionally omitted
}

// ============================================================================
// CONTENT LOADING STRUCTURES
// ============================================================================

/// Status of a content loading job
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum LoadingStatus {
    InProgress,
    Completed,
    Failed,
    Paused,
}

/// A loading job for resilient content loading
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LoadingJob {
    pub proposal_id: u64,
    pub staging_canister: Principal,
    pub staging_path: String,
    pub content_hash: String,
    pub total_units: u32,
    pub loaded_units: u32,
    pub status: LoadingStatus,
    pub last_error: Option<String>,
    pub started_at: u64,
    pub updated_at: u64,
}

impl Storable for LoadingJob {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode LoadingJob")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1000,
        is_fixed_size: false,
    };
}

// ============================================================================
// VERSION HISTORY STRUCTURES
// ============================================================================

/// Type of change made to content
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum ChangeType {
    Created,
    Updated,
    Deleted,
}

/// Key for version history: (content_id, version_number)
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VersionKey {
    pub content_id: String,
    pub version: u64,
}

impl Storable for VersionKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode VersionKey")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 500,
        is_fixed_size: false,
    };
}

/// A snapshot of content before modification
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ContentSnapshot {
    pub content: ContentNode,
    pub modified_at: u64,
    pub modified_by_proposal: u64,
    pub change_type: ChangeType,
}

impl Storable for ContentSnapshot {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ContentSnapshot")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100000,
        is_fixed_size: false,
    };
}

// ============================================================================
// CHILDREN INDEX - Vec wrapper for stable storage
// ============================================================================

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
struct ChildrenList(Vec<String>);

impl Storable for ChildrenList {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ChildrenList")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 50000,
        is_fixed_size: false,
    };
}

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================
// 
// Memory IDs:
//   0  - STAKING_HUB_ID: Configuration
//   1  - GOVERNANCE_CANISTER_ID: Governance canister
//   2  - CONTENT_NODES: Flexible content storage
//   3  - CHILDREN_INDEX: Parent-to-children mapping
//   4  - QUIZ_INDEX: Quiz cache for O(1) lookup
//   5  - GLOBAL_QUIZ_CONFIG: Single config for all quizzes
//   6  - CONTENT_VERSION: Global version counter
//   7  - LOADING_JOBS: Resilient content loading jobs
//   8  - VERSION_HISTORY: Content change history
//   9  - CONTENT_VERSIONS: Per-content version tracking

thread_local! {
    /// Memory manager for allocating virtual memory regions
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    /// Principal ID of the staking hub canister
    static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Principal ID of the governance canister
    static GOVERNANCE_CANISTER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Principal::anonymous()
        ).unwrap()
    );

    /// All content nodes - the flexible tree structure
    static CONTENT_NODES: RefCell<StableBTreeMap<String, ContentNode, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    /// Index: parent_id -> list of child IDs (for tree traversal)
    static CHILDREN_INDEX: RefCell<StableBTreeMap<String, ChildrenList, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );

    /// Quiz index: content_id -> quiz cache data (for O(1) lookup)
    static QUIZ_INDEX: RefCell<StableBTreeMap<String, QuizCacheData, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    /// GLOBAL quiz config - ONE config for ALL quizzes
    static GLOBAL_QUIZ_CONFIG: RefCell<StableCell<QuizConfig, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            QuizConfig::default()
        ).unwrap()
    );

    /// Global content version (increments on any change)
    static CONTENT_VERSION: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            0
        ).unwrap()
    );

    /// Loading jobs (for resilient content loading)
    static LOADING_JOBS: RefCell<StableBTreeMap<u64, LoadingJob, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7)))
        )
    );

    /// Version history for audit trail
    static VERSION_HISTORY: RefCell<StableBTreeMap<VersionKey, ContentSnapshot, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8)))
        )
    );

    /// Per-content version tracking: content_id -> current version
    static CONTENT_VERSIONS: RefCell<StableBTreeMap<String, u64, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9)))
        )
    );
}

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
// HELPER FUNCTIONS
// ============================================================================

/// Stable deterministic hash for answer verification (djb2 variant)
fn stable_hash(data: &[u8]) -> [u8; 32] {
    let mut hash: u64 = 5381;
    for b in data {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*b as u64);
    }
    
    // Expand 8-byte hash to 32 bytes
    let b = hash.to_le_bytes();
    let mut res = [0u8; 32];
    res[0..8].copy_from_slice(&b);
    res[8..16].copy_from_slice(&b);
    res[16..24].copy_from_slice(&b);
    res[24..32].copy_from_slice(&b);
    res
}

/// Helper to rebuild quiz index (useful after upgrades or hash changes)
fn rebuild_quiz_index() {
    // 1. Collect all nodes that have quizzes (in separate scope to avoid borrow hold)
    let nodes_with_quizzes: Vec<ContentNode> = CONTENT_NODES.with(|c| {
        c.borrow().iter()
            .map(|(_, node)| node)
            .filter(|n| n.quiz.is_some())
            .collect()
    });

    // 2. Repopulate QUIZ_INDEX
    QUIZ_INDEX.with(|q| {
        let mut idx = q.borrow_mut();
        for node in nodes_with_quizzes {
            if let Some(quiz) = node.quiz {
                 let cache_data = QuizCacheData {
                    content_id: node.id.clone(),
                    answer_hashes: quiz.questions.iter()
                        .map(|q| stable_hash(&q.answer.to_le_bytes()))
                        .collect(),
                    question_count: quiz.questions.len() as u8,
                    version: node.version, 
                };
                idx.insert(node.id, cache_data);
            }
        }
    });
}

/// Increment global content version
fn increment_version() -> u64 {
    CONTENT_VERSION.with(|v| {
        let mut cell = v.borrow_mut();
        let current = *cell.get();
        let new_version = current + 1;
        cell.set(new_version).expect("Failed to increment version");
        new_version
    })
}

/// Get content version for a specific node
fn get_content_version(content_id: &str) -> u64 {
    CONTENT_VERSIONS.with(|v| v.borrow().get(&content_id.to_string()).unwrap_or(0))
}

/// Set content version for a specific node
fn set_content_version(content_id: &str, version: u64) {
    CONTENT_VERSIONS.with(|v| {
        v.borrow_mut().insert(content_id.to_string(), version);
    });
}

/// Convert internal ContentNode to public version (without answers)
fn to_public_node(node: &ContentNode) -> PublicContentNode {
    PublicContentNode {
        id: node.id.clone(),
        parent_id: node.parent_id.clone(),
        order: node.order,
        display_type: node.display_type.clone(),
        title: node.title.clone(),
        description: node.description.clone(),
        content: node.content.clone(),
        paraphrase: node.paraphrase.clone(),
        media: node.media.clone(),
        quiz: node.quiz.as_ref().map(|q| PublicQuizData {
            questions: q.questions.iter().map(|question| PublicQuizQuestion {
                question: question.question.clone(),
                options: question.options.clone(),
            }).collect(),
        }),
        created_at: node.created_at,
        updated_at: node.updated_at,
        version: node.version,
    }
}

// ============================================================================
// CONTENT NODE MANAGEMENT
// ============================================================================

/// Add or update a content node (internal function)
fn add_content_node_internal(node: ContentNode, proposal_id: Option<u64>) -> Result<(), String> {
    let id = node.id.clone();
    let now = ic_cdk::api::time();
    
    // Check if this is an update
    let is_update = CONTENT_NODES.with(|c| c.borrow().contains_key(&id));
    
    // If updating, save snapshot for version history
    if is_update {
        if let Some(prop_id) = proposal_id {
            CONTENT_NODES.with(|c| {
                if let Some(old_node) = c.borrow().get(&id) {
                    let current_version = get_content_version(&id);
                    VERSION_HISTORY.with(|h| {
                        h.borrow_mut().insert(
                            VersionKey { content_id: id.clone(), version: current_version },
                            ContentSnapshot {
                                content: old_node.clone(),
                                modified_at: now,
                                modified_by_proposal: prop_id,
                                change_type: ChangeType::Updated,
                            }
                        );
                    });
                }
            });
        }
    } else if let Some(prop_id) = proposal_id {
        // New node - save creation snapshot
        VERSION_HISTORY.with(|h| {
            h.borrow_mut().insert(
                VersionKey { content_id: id.clone(), version: 1 },
                ContentSnapshot {
                    content: node.clone(),
                    modified_at: now,
                    modified_by_proposal: prop_id,
                    change_type: ChangeType::Created,
                }
            );
        });
    }
    
    // 1. Add to main content map - O(1)
    CONTENT_NODES.with(|c| c.borrow_mut().insert(id.clone(), node.clone()));
    
    // 2. Update children index - O(1)
    if let Some(ref parent_id) = node.parent_id {
        CHILDREN_INDEX.with(|idx| {
            let mut index = idx.borrow_mut();
            let mut children = index.get(parent_id).unwrap_or_default();
            if !children.0.contains(&id) {
                children.0.push(id.clone());
                index.insert(parent_id.clone(), children);
            }
        });
    }
    
    // 3. If has quiz, update quiz index - O(1)
    if let Some(ref quiz) = node.quiz {
        let cache_data = QuizCacheData {
            content_id: id.clone(),
            answer_hashes: quiz.questions.iter()
                .map(|q| stable_hash(&q.answer.to_le_bytes())) // Updated to stable_hash
                .collect(),
            question_count: quiz.questions.len() as u8,
            version: increment_version(),
        };
        QUIZ_INDEX.with(|q| q.borrow_mut().insert(id.clone(), cache_data));
    } else {
        // Remove from quiz index if quiz was removed
        QUIZ_INDEX.with(|q| q.borrow_mut().remove(&id));
    }
    
    // 4. Update version tracking
    let new_version = if is_update {
        get_content_version(&id) + 1
    } else {
        1
    };
    set_content_version(&id, new_version);

    // 5. Push cache to Hub (if not batch loading i.e. proposal_id is None)
    if proposal_id.is_none() {
        // Retrieve fresh copy to push
        let maybe_cache = QUIZ_INDEX.with(|q| q.borrow().get(&id));
        if let Some(cache_data) = maybe_cache {
            let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
             if hub_id != Principal::anonymous() {
                 let unit_id = id.clone();
                 ic_cdk::spawn(async move {
                     let _ = ic_cdk::call::<_, ()>(
                         hub_id,
                         "distribute_quiz_cache",
                         (unit_id, cache_data)
                     ).await;
                 });
             }
        }
    }
    
    Ok(())
}

/// Add or update a content node (public function - admin only)
#[update]
fn add_content_node(node: ContentNode) -> Result<(), String> {
    // In production, verify caller is admin or governance canister
    add_content_node_internal(node, None)
}

/// Add multiple content nodes (batch operation)
#[update]
fn add_content_nodes(nodes: Vec<ContentNode>) -> Result<u32, String> {
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
fn delete_content_node(id: String, proposal_id: u64) -> Result<(), String> {
    let now = ic_cdk::api::time();
    
    // Get the node before deleting
    let node = CONTENT_NODES.with(|c| c.borrow().get(&id))
        .ok_or("Node not found")?;
    
    // Check if has children
    let has_children = CHILDREN_INDEX.with(|idx| {
        idx.borrow().get(&id).map(|c| !c.0.is_empty()).unwrap_or(false)
    });
    
    if has_children {
        return Err("Cannot delete node with children".to_string());
    }
    
    // Save deletion snapshot
    let current_version = get_content_version(&id);
    VERSION_HISTORY.with(|h| {
        h.borrow_mut().insert(
            VersionKey { content_id: id.clone(), version: current_version + 1 },
            ContentSnapshot {
                content: node.clone(),
                modified_at: now,
                modified_by_proposal: proposal_id,
                change_type: ChangeType::Deleted,
            }
        );
    });
    
    // Remove from parent's children
    if let Some(ref parent_id) = node.parent_id {
        CHILDREN_INDEX.with(|idx| {
            let mut index = idx.borrow_mut();
            if let Some(mut children) = index.get(parent_id) {
                children.0.retain(|child_id| child_id != &id);
                index.insert(parent_id.clone(), children);
            }
        });
    }
    
    // Remove from quiz index
    QUIZ_INDEX.with(|q| q.borrow_mut().remove(&id));
    
    // Remove from content nodes
    CONTENT_NODES.with(|c| c.borrow_mut().remove(&id));
    
    increment_version();
    
    Ok(())
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

/// Get global quiz config
#[query]
fn get_global_quiz_config() -> QuizConfig {
    GLOBAL_QUIZ_CONFIG.with(|c| c.borrow().get().clone())
}

/// Update global quiz config (governance only)
/// After updating, distributes the new config to all user_profile shards
#[update]
async fn update_global_quiz_config(
    new_reward_amount: Option<u64>,
    new_pass_threshold: Option<u8>,
    new_max_attempts: Option<u8>,
    new_max_daily_quizzes: Option<u8>,
    new_max_weekly_quizzes: Option<u8>,
    new_max_monthly_quizzes: Option<u8>,
    new_max_yearly_quizzes: Option<u16>,
) -> Result<(), String> {
    // In production, verify caller is governance canister
    
    let updated_config = GLOBAL_QUIZ_CONFIG.with(|c| {
        let mut config = c.borrow().get().clone();
        
        if let Some(reward) = new_reward_amount {
            config.reward_amount = reward;
        }
        if let Some(threshold) = new_pass_threshold {
            if threshold > 100 {
                return Err("Pass threshold cannot exceed 100%".to_string());
            }
            config.pass_threshold_percent = threshold;
        }
        if let Some(attempts) = new_max_attempts {
            config.max_daily_attempts = attempts;
        }
        
        // Quiz limits
        if let Some(daily) = new_max_daily_quizzes {
            config.max_daily_quizzes = daily;
        }
        if let Some(weekly) = new_max_weekly_quizzes {
            config.max_weekly_quizzes = weekly;
        }
        if let Some(monthly) = new_max_monthly_quizzes {
            config.max_monthly_quizzes = monthly;
        }
        if let Some(yearly) = new_max_yearly_quizzes {
            config.max_yearly_quizzes = yearly;
        }
        
        c.borrow_mut().set(config.clone()).expect("Failed to update quiz config");
        Ok(config)
    })?;
    
    // Distribute to all shards via staking_hub
    let hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    // Convert to cached format for distribution
    let cached_config = CachedQuizConfig {
        reward_amount: updated_config.reward_amount,
        pass_threshold_percent: updated_config.pass_threshold_percent,
        max_daily_attempts: updated_config.max_daily_attempts,
        max_daily_quizzes: updated_config.max_daily_quizzes,
        max_weekly_quizzes: updated_config.max_weekly_quizzes,
        max_monthly_quizzes: updated_config.max_monthly_quizzes,
        max_yearly_quizzes: updated_config.max_yearly_quizzes,
        version: ic_cdk::api::time() / 1_000_000_000, // Use timestamp as version
    };
    
    // Fire and forget - don't block on distribution
    ic_cdk::spawn(async move {
        let _: Result<(Result<u64, String>,), _> = ic_cdk::call(
            hub_id,
            "distribute_quiz_config",
            (cached_config,)
        ).await;
    });
    
    Ok(())
}

/// Verify quiz answers (called by user_profile shards or directly)
#[query]
fn verify_quiz(content_id: String, answers: Vec<u8>) -> (bool, u64, u64) {
    if let Some(cache) = QUIZ_INDEX.with(|q| q.borrow().get(&content_id)) {
        if cache.question_count as usize != answers.len() {
            return (false, 0, cache.question_count as u64);
        }
        
        let mut correct = 0u64;
        for (i, answer) in answers.iter().enumerate() {
            let answer_hash = stable_hash(&answer.to_le_bytes());
            if answer_hash == cache.answer_hashes[i] {
                correct += 1;
            }
        }
        
        let config = get_global_quiz_config();
        let passed = if cache.question_count > 0 {
            (correct * 100) / cache.question_count as u64 >= config.pass_threshold_percent as u64
        } else {
            false
        };
        
        return (passed, correct, cache.question_count as u64);
    }
    
    (false, 0, 0)
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
    let now = ic_cdk::api::time();
    
    // Create loading job in STABLE storage
    let job = LoadingJob {
        proposal_id,
        staging_canister,
        staging_path,
        content_hash,
        total_units,
        loaded_units: 0,
        status: LoadingStatus::InProgress,
        last_error: None,
        started_at: now,
        updated_at: now,
    };
    
    LOADING_JOBS.with(|jobs| jobs.borrow_mut().insert(proposal_id, job));
    
    // Start processing (will self-call to continue)
    continue_loading(proposal_id).await
}

/// Continue loading (self-call pattern for resilience)
#[update]
async fn continue_loading(proposal_id: u64) -> Result<(), String> {
    const BATCH_SIZE: u32 = 10;
    
    let job = LOADING_JOBS.with(|jobs| jobs.borrow().get(&proposal_id))
        .ok_or("Loading job not found")?;
    
    if job.status != LoadingStatus::InProgress {
        return Ok(());
    }
    
    // Fetch next batch from staging
    let result: Result<(Vec<ContentNode>,), _> = ic_cdk::call(
        job.staging_canister,
        "get_content_chunk",
        (job.content_hash.clone(), job.loaded_units, BATCH_SIZE)
    ).await;
    
    match result {
        Ok((batch,)) => {
            // Process each node
            for node in &batch {
                if let Err(e) = add_content_node_internal(node.clone(), Some(proposal_id)) {
                    // Save error and pause - get job in separate scope
                    let error_job = LOADING_JOBS.with(|jobs| {
                        jobs.borrow().get(&proposal_id).map(|j| {
                            let mut updated = j.clone();
                            updated.status = LoadingStatus::Paused;
                            updated.last_error = Some(e.clone());
                            updated.updated_at = ic_cdk::api::time();
                            updated
                        })
                    });
                    if let Some(job_to_insert) = error_job {
                        LOADING_JOBS.with(|jobs| {
                            jobs.borrow_mut().insert(proposal_id, job_to_insert);
                        });
                    }
                    return Err(e);
                }
            }
            
            // Update progress
            let new_loaded = job.loaded_units + batch.len() as u32;
            let is_complete = new_loaded >= job.total_units || batch.is_empty();
            
            // Get the job, clone it, then update in separate scope
            let updated_job = LOADING_JOBS.with(|jobs| {
                jobs.borrow().get(&proposal_id).map(|j| {
                    let mut updated = j.clone();
                    updated.loaded_units = new_loaded;
                    updated.status = if is_complete { LoadingStatus::Completed } else { LoadingStatus::InProgress };
                    updated.updated_at = ic_cdk::api::time();
                    updated
                })
            });
            
            // Insert in separate scope
            if let Some(job_to_insert) = updated_job {
                LOADING_JOBS.with(|jobs| {
                    jobs.borrow_mut().insert(proposal_id, job_to_insert);
                });
            }
            
            if is_complete {
                // Mark content as loaded in staging canister
                let staging_canister = job.staging_canister;
                let content_hash = job.content_hash.clone();
                let _ = ic_cdk::call::<_, (Result<(), String>,)>(
                    staging_canister,
                    "mark_loaded",
                    (content_hash,)
                ).await;
            } else {
                // Continue with self-call
                let self_id = ic_cdk::api::id();
                let _ = ic_cdk::call::<_, ()>(self_id, "continue_loading", (proposal_id,)).await;
            }
            
            Ok(())
        }
        Err((code, msg)) => {
            let error = format!("Failed to fetch from staging: {:?} {}", code, msg);
            let error_job = LOADING_JOBS.with(|jobs| {
                jobs.borrow().get(&proposal_id).map(|j| {
                    let mut updated = j.clone();
                    updated.status = LoadingStatus::Paused;
                    updated.last_error = Some(error.clone());
                    updated.updated_at = ic_cdk::api::time();
                    updated
                })
            });
            if let Some(job_to_insert) = error_job {
                LOADING_JOBS.with(|jobs| {
                    jobs.borrow_mut().insert(proposal_id, job_to_insert);
                });
            }
            Err(error)
        }
    }
}

/// Resume loading after error or pause
#[update]
async fn resume_loading(proposal_id: u64) -> Result<(), String> {
    let resume_job = LOADING_JOBS.with(|jobs| {
        jobs.borrow().get(&proposal_id).map(|j| {
            let mut updated = j.clone();
            updated.status = LoadingStatus::InProgress;
            updated.last_error = None;
            updated.updated_at = ic_cdk::api::time();
            updated
        })
    });
    if let Some(job_to_insert) = resume_job {
        LOADING_JOBS.with(|jobs| {
            jobs.borrow_mut().insert(proposal_id, job_to_insert);
        });
    }
    
    continue_loading(proposal_id).await
}

/// Resume incomplete jobs after upgrade
async fn resume_incomplete_jobs() {
    let incomplete: Vec<u64> = LOADING_JOBS.with(|jobs| {
        jobs.borrow()
            .iter()
            .filter(|(_, job)| job.status == LoadingStatus::InProgress)
            .map(|(id, _)| id)
            .collect()
    });
    
    for proposal_id in incomplete {
        let _ = continue_loading(proposal_id).await;
    }
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
