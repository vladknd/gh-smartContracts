use candid::Principal;
use sha2::{Sha256, Digest};
use crate::state::*;
use crate::types::*;
use crate::constants::{MAX_CHUNK_SIZE, MAX_FILE_SIZE};

/// Encode bytes to hex
pub mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

// Helpers
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn generate_session_id(caller: Principal, now: u64) -> String {
    let combined = format!("{}{}", now, caller);
    compute_hash(combined.as_bytes())[..16].to_string()
}

pub fn is_allowed_uploader(caller: &Principal) -> bool {
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

// Logic impls
pub fn start_upload_impl(
    caller: Principal,
    now: u64,
    filename: String,
    content_type: String,
    media_type: MediaType,
    expected_size: u64,
) -> Result<String, String> {
    if !is_allowed_uploader(&caller) {
        return Err("Not authorized to upload".to_string());
    }
    
    if expected_size > MAX_FILE_SIZE {
        return Err(format!("File too large. Max size: {} bytes", MAX_FILE_SIZE));
    }
    
    let session_id = generate_session_id(caller, now);
    
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

pub fn upload_chunk_impl(caller: Principal, session_id: String, chunk_index: u32, data: Vec<u8>) -> Result<(), String> {
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

pub fn finalize_upload_impl(caller: Principal, now: u64, session_id: String) -> Result<String, String> {
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
        // Re-insert session on error to allow retry/completion
        UPLOAD_SESSIONS.with(|s| s.borrow_mut().insert(session_id.clone(), session.clone()));
        
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
        // Re-insert
        UPLOAD_SESSIONS.with(|s| s.borrow_mut().insert(session_id.clone(), session.clone()));
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
        uploaded_at: now,
        chunk_count,
    };
    
    FILE_METADATA.with(|m| m.borrow_mut().insert(hash.clone(), metadata));
    
    Ok(hash)
}

pub fn upload_file_impl(
    caller: Principal,
    now: u64,
    filename: String,
    content_type: String,
    media_type: MediaType,
    data: Vec<u8>,
) -> Result<String, String> {
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
        uploaded_at: now,
        chunk_count: 1,
    };
    
    FILE_METADATA.with(|m| m.borrow_mut().insert(hash.clone(), metadata));
    
    Ok(hash)
}

pub fn cancel_upload_impl(caller: Principal, session_id: String) -> Result<(), String> {
    let session = UPLOAD_SESSIONS.with(|s| s.borrow().get(&session_id))
        .ok_or("Upload session not found")?;
    
    if session.uploader != caller && !ic_cdk::api::is_controller(&caller) {
        return Err("Not authorized to cancel this upload".to_string());
    }
    
    UPLOAD_SESSIONS.with(|s| s.borrow_mut().remove(&session_id));
    
    Ok(())
}
