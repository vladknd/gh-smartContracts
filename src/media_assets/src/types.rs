use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use std::borrow::Cow;
use ic_stable_structures::{Storable, DefaultMemoryImpl};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::storable::Bound;
use crate::constants::{MAX_CHUNK_SIZE, MAX_FILE_SIZE};

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
pub struct InitArgs {
    /// List of principals allowed to upload (empty = anyone can upload)
    pub allowed_uploaders: Vec<Principal>,
}

/// Media type enum
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum MediaType {
    Video,
    Audio,
    Image,
    PDF,
    Other,
}

/// Metadata about an uploaded file
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FileMetadata {
    /// SHA256 hash of the file content (also the key)
    pub hash: String,
    /// Original filename
    pub filename: String,
    /// MIME type (e.g., "video/mp4", "image/png")
    pub content_type: String,
    /// Media type category
    pub media_type: MediaType,
    /// File size in bytes
    pub size: u64,
    /// Who uploaded it
    pub uploader: Principal,
    /// When it was uploaded (nanoseconds)
    pub uploaded_at: u64,
    /// Number of chunks
    pub chunk_count: u32,
}

impl Storable for FileMetadata {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode FileMetadata")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1000,
        is_fixed_size: false,
    };
}

/// A chunk of file data
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FileChunk {
    pub data: Vec<u8>,
}

impl Storable for FileChunk {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode FileChunk")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_CHUNK_SIZE as u32 + 100, // Data + overhead
        is_fixed_size: false,
    };
}

/// Key for chunk storage: (file_hash, chunk_index)
#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkKey {
    pub file_hash: String,
    pub chunk_index: u32,
}

impl Storable for ChunkKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ChunkKey")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

/// Upload session for tracking multi-chunk uploads
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UploadSession {
    pub session_id: String,
    pub filename: String,
    pub content_type: String,
    pub media_type: MediaType,
    pub expected_size: u64,
    pub uploaded_size: u64,
    pub uploader: Principal,
    pub started_at: u64,
    pub chunks_received: Vec<u32>,
    pub temp_chunks: Vec<Vec<u8>>,
}

impl Storable for UploadSession {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode UploadSession")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_FILE_SIZE as u32 + 10000, // Full file + metadata
        is_fixed_size: false,
    };
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
