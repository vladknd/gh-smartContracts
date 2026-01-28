use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use std::borrow::Cow;
use ic_stable_structures::{Storable, DefaultMemoryImpl};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::storable::Bound;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

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

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    /// Governance canister that can read and delete staged content
    pub governance_canister_id: Principal,
    /// Learning engine that can read staged content
    pub learning_engine_id: Principal,
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

/// List wrapper for stable storage
#[derive(CandidType, Deserialize, Clone, Debug, Default)]
pub struct PrincipalList(pub Vec<Principal>);

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
