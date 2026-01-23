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
// MEDIA ASSETS CANISTER
// ============================================================================
// 
// This canister provides permanent storage for media files (videos, images, 
// audio, PDFs). Files are stored with content-addressed hashing for integrity.
// 
// Key features:
// - Chunked upload for large files
// - Content-addressed storage (SHA256 hash as key)
// - Immutable once uploaded (no delete/update)
// - HTTP asset serving for frontend access

// ============================================================================
// CONSTANTS
// ============================================================================

/// Maximum chunk size for uploads (2MB)
const MAX_CHUNK_SIZE: usize = 2 * 1024 * 1024;

/// Maximum file size (100MB)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
struct InitArgs {
    /// List of principals allowed to upload (empty = anyone can upload)
    allowed_uploaders: Vec<Principal>,
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
struct ChunkKey {
    file_hash: String,
    chunk_index: u32,
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

    /// File metadata: hash -> FileMetadata
    static FILE_METADATA: RefCell<StableBTreeMap<String, FileMetadata, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    /// File chunks: (hash, index) -> chunk data
    static FILE_CHUNKS: RefCell<StableBTreeMap<ChunkKey, FileChunk, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    /// Active upload sessions
    static UPLOAD_SESSIONS: RefCell<StableBTreeMap<String, UploadSession, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    /// Allowed uploaders (empty = anyone)
    static ALLOWED_UPLOADERS: RefCell<StableCell<PrincipalList, Memory>> = RefCell::new(
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
    ALLOWED_UPLOADERS.with(|u| {
        u.borrow_mut().set(PrincipalList(args.allowed_uploaders))
            .expect("Failed to set allowed uploaders");
    });
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Check if caller is allowed to upload
fn is_allowed_uploader(caller: &Principal) -> bool {
    // Controllers always allowed
    if ic_cdk::api::is_controller(caller) {
        return true;
    }
    
    ALLOWED_UPLOADERS.with(|u| {
        let list = u.borrow().get().clone();
        // Empty list = anyone can upload
        list.0.is_empty() || list.0.contains(caller)
    })
}

/// Compute SHA256 hash of data
fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Generate session ID
fn generate_session_id() -> String {
    let now = ic_cdk::api::time();
    let caller = ic_cdk::caller();
    let combined = format!("{}{}", now, caller);
    compute_hash(combined.as_bytes())[..16].to_string()
}

/// Encode bytes to hex
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

// ============================================================================
// UPLOAD FUNCTIONS
// ============================================================================

/// Start a new upload session
#[update]
fn start_upload(
    filename: String,
    content_type: String,
    media_type: MediaType,
    expected_size: u64,
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    if !is_allowed_uploader(&caller) {
        return Err("Not authorized to upload".to_string());
    }
    
    if expected_size > MAX_FILE_SIZE {
        return Err(format!("File too large. Max size: {} bytes", MAX_FILE_SIZE));
    }
    
    let session_id = generate_session_id();
    let now = ic_cdk::api::time();
    
    let session = UploadSession {
        session_id: session_id.clone(),
        filename,
        content_type,
        media_type,
        expected_size,
        uploaded_size: 0,
        uploader: caller,
        started_at: now,
        chunks_received: Vec::new(),
        temp_chunks: Vec::new(),
    };
    
    UPLOAD_SESSIONS.with(|s| s.borrow_mut().insert(session_id.clone(), session));
    
    Ok(session_id)
}

/// Upload a chunk of data
#[update]
fn upload_chunk(session_id: String, chunk_index: u32, data: Vec<u8>) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    if data.len() > MAX_CHUNK_SIZE {
        return Err(format!("Chunk too large. Max size: {} bytes", MAX_CHUNK_SIZE));
    }
    
    let mut session = UPLOAD_SESSIONS.with(|s| s.borrow().get(&session_id))
        .ok_or("Upload session not found")?;
    
    if session.uploader != caller {
        return Err("Not the session owner".to_string());
    }
    
    if session.chunks_received.contains(&chunk_index) {
        return Err("Chunk already uploaded".to_string());
    }
    
    // Add chunk to temp storage
    session.uploaded_size += data.len() as u64;
    session.chunks_received.push(chunk_index);
    
    // Ensure temp_chunks vector is large enough
    while session.temp_chunks.len() <= chunk_index as usize {
        session.temp_chunks.push(Vec::new());
    }
    session.temp_chunks[chunk_index as usize] = data;
    
    UPLOAD_SESSIONS.with(|s| s.borrow_mut().insert(session_id, session));
    
    Ok(())
}

/// Finalize upload and get the file hash
#[update]
fn finalize_upload(session_id: String) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    let session = UPLOAD_SESSIONS.with(|s| s.borrow_mut().remove(&session_id))
        .ok_or("Upload session not found")?;
    
    if session.uploader != caller {
        // Re-insert session since caller doesn't own it
        UPLOAD_SESSIONS.with(|s| s.borrow_mut().insert(session_id, session));
        return Err("Not the session owner".to_string());
    }
    
    // Verify all chunks received
    let expected_chunks = ((session.expected_size + MAX_CHUNK_SIZE as u64 - 1) / MAX_CHUNK_SIZE as u64) as u32;
    if session.chunks_received.len() != expected_chunks as usize {
        return Err(format!(
            "Missing chunks. Expected: {}, Received: {}",
            expected_chunks,
            session.chunks_received.len()
        ));
    }
    
    // Reconstruct file and compute hash
    let mut full_data = Vec::with_capacity(session.expected_size as usize);
    for chunk in &session.temp_chunks {
        full_data.extend_from_slice(chunk);
    }
    
    // Verify size
    if full_data.len() as u64 != session.expected_size {
        return Err(format!(
            "Size mismatch. Expected: {}, Got: {}",
            session.expected_size,
            full_data.len()
        ));
    }
    
    // Compute hash
    let hash = compute_hash(&full_data);
    
    // Check if file already exists (deduplication)
    if FILE_METADATA.with(|m| m.borrow().contains_key(&hash)) {
        // File already exists, just return the hash
        return Ok(hash);
    }
    
    // Store chunks
    let chunk_count = session.temp_chunks.len() as u32;
    for (i, chunk_data) in session.temp_chunks.into_iter().enumerate() {
        let key = ChunkKey {
            file_hash: hash.clone(),
            chunk_index: i as u32,
        };
        FILE_CHUNKS.with(|c| c.borrow_mut().insert(key, FileChunk { data: chunk_data }));
    }
    
    // Store metadata
    let metadata = FileMetadata {
        hash: hash.clone(),
        filename: session.filename,
        content_type: session.content_type,
        media_type: session.media_type,
        size: full_data.len() as u64,
        uploader: session.uploader,
        uploaded_at: ic_cdk::api::time(),
        chunk_count,
    };
    
    FILE_METADATA.with(|m| m.borrow_mut().insert(hash.clone(), metadata));
    
    Ok(hash)
}

/// Simple single-chunk upload for small files
#[update]
fn upload_file(
    filename: String,
    content_type: String,
    media_type: MediaType,
    data: Vec<u8>,
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    if !is_allowed_uploader(&caller) {
        return Err("Not authorized to upload".to_string());
    }
    
    if data.len() > MAX_FILE_SIZE as usize {
        return Err(format!("File too large. Max size: {} bytes", MAX_FILE_SIZE));
    }
    
    let hash = compute_hash(&data);
    
    // Check if already exists (deduplication)
    if FILE_METADATA.with(|m| m.borrow().contains_key(&hash)) {
        return Ok(hash);
    }
    
    // Store as single chunk
    let key = ChunkKey {
        file_hash: hash.clone(),
        chunk_index: 0,
    };
    FILE_CHUNKS.with(|c| c.borrow_mut().insert(key, FileChunk { data: data.clone() }));
    
    // Store metadata
    let metadata = FileMetadata {
        hash: hash.clone(),
        filename,
        content_type,
        media_type,
        size: data.len() as u64,
        uploader: caller,
        uploaded_at: ic_cdk::api::time(),
        chunk_count: 1,
    };
    
    FILE_METADATA.with(|m| m.borrow_mut().insert(hash.clone(), metadata));
    
    Ok(hash)
}

/// Cancel an upload session
#[update]
fn cancel_upload(session_id: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    let session = UPLOAD_SESSIONS.with(|s| s.borrow().get(&session_id))
        .ok_or("Upload session not found")?;
    
    if session.uploader != caller && !ic_cdk::api::is_controller(&caller) {
        return Err("Not authorized to cancel this upload".to_string());
    }
    
    UPLOAD_SESSIONS.with(|s| s.borrow_mut().remove(&session_id));
    
    Ok(())
}

// ============================================================================
// QUERY FUNCTIONS
// ============================================================================

/// Get file metadata by hash
#[query]
fn get_file_metadata(hash: String) -> Option<FileMetadata> {
    FILE_METADATA.with(|m| m.borrow().get(&hash))
}

/// Get a specific chunk of a file
#[query]
fn get_file_chunk(hash: String, chunk_index: u32) -> Option<Vec<u8>> {
    let key = ChunkKey { file_hash: hash, chunk_index };
    FILE_CHUNKS.with(|c| c.borrow().get(&key).map(|chunk| chunk.data))
}

/// Get entire file (for small files only)
#[query]
fn get_file(hash: String) -> Result<Vec<u8>, String> {
    let metadata = FILE_METADATA.with(|m| m.borrow().get(&hash))
        .ok_or("File not found")?;
    
    // Limit to prevent too much data in single response
    if metadata.size > 2 * 1024 * 1024 {
        return Err("File too large for single query. Use get_file_chunk instead.".to_string());
    }
    
    let mut data = Vec::new();
    for i in 0..metadata.chunk_count {
        let key = ChunkKey { file_hash: hash.clone(), chunk_index: i };
        let chunk = FILE_CHUNKS.with(|c| c.borrow().get(&key))
            .ok_or("Missing chunk")?;
        data.extend_from_slice(&chunk.data);
    }
    
    Ok(data)
}

/// Get URL for a file (returns the asset URL path)
#[query]
fn get_file_url(hash: String) -> Result<String, String> {
    if !FILE_METADATA.with(|m| m.borrow().contains_key(&hash)) {
        return Err("File not found".to_string());
    }
    
    // Return the URL path that would be served via HTTP
    let canister_id = ic_cdk::api::id();
    Ok(format!("https://{}.raw.icp0.io/file/{}", canister_id, hash))
}

/// Check if a file exists
#[query]
fn file_exists(hash: String) -> bool {
    FILE_METADATA.with(|m| m.borrow().contains_key(&hash))
}

/// Get upload session status
#[query]
fn get_upload_session(session_id: String) -> Option<UploadSession> {
    UPLOAD_SESSIONS.with(|s| s.borrow().get(&session_id))
}

/// Get all file hashes (paginated)
#[query]
fn list_files(offset: u64, limit: u64) -> Vec<FileMetadata> {
    FILE_METADATA.with(|m| {
        m.borrow()
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .map(|(_, metadata)| metadata)
            .collect()
    })
}

/// Get file count
#[query]
fn get_file_count() -> u64 {
    FILE_METADATA.with(|m| m.borrow().len())
}

/// Get total storage used
#[query]
fn get_total_storage() -> u64 {
    FILE_METADATA.with(|m| {
        m.borrow().iter().map(|(_, meta)| meta.size).sum()
    })
}

// ============================================================================
// ADMIN FUNCTIONS
// ============================================================================

/// Add an allowed uploader (controller only)
#[update]
fn add_allowed_uploader(principal: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Only controllers can add uploaders".to_string());
    }
    
    ALLOWED_UPLOADERS.with(|u| {
        let mut list = u.borrow().get().clone();
        if !list.0.contains(&principal) {
            list.0.push(principal);
            u.borrow_mut().set(list).expect("Failed to update uploaders");
        }
    });
    
    Ok(())
}

/// Remove an allowed uploader (controller only)
#[update]
fn remove_allowed_uploader(principal: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Only controllers can remove uploaders".to_string());
    }
    
    ALLOWED_UPLOADERS.with(|u| {
        let mut list = u.borrow().get().clone();
        list.0.retain(|p| p != &principal);
        u.borrow_mut().set(list).expect("Failed to update uploaders");
    });
    
    Ok(())
}

/// Get list of allowed uploaders
#[query]
fn get_allowed_uploaders() -> Vec<Principal> {
    ALLOWED_UPLOADERS.with(|u| u.borrow().get().0.clone())
}

ic_cdk::export_candid!();
