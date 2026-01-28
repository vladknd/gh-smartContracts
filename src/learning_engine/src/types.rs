use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    /// Principal ID of the staking hub (for future use)
    pub staking_hub_id: Principal,
    /// Principal ID of the governance canister
    pub governance_canister_id: Option<Principal>,
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
/// NOTE: Configuration is now managed by the Staking Hub
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QuizData {
    /// Quiz questions
    pub questions: Vec<QuizQuestion>,
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


#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct ChildrenList(pub Vec<String>);

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
