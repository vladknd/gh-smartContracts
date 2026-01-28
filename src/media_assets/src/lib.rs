//! # Media Assets Canister
//! 
//! This canister provides permanent storage for media files (videos, images, 
//! audio, PDFs). Files are stored with content-addressed hashing for integrity.

mod constants;
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
    ALLOWED_UPLOADERS.with(|u| {
        u.borrow_mut().set(PrincipalList(args.allowed_uploaders))
            .expect("Failed to set allowed uploaders");
    });
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
    service::start_upload_impl(ic_cdk::caller(), ic_cdk::api::time(), filename, content_type, media_type, expected_size)
}

/// Upload a chunk of data
#[update]
fn upload_chunk(session_id: String, chunk_index: u32, data: Vec<u8>) -> Result<(), String> {
    service::upload_chunk_impl(ic_cdk::caller(), session_id, chunk_index, data)
}

/// Finalize upload and get the file hash
#[update]
fn finalize_upload(session_id: String) -> Result<String, String> {
    service::finalize_upload_impl(ic_cdk::caller(), ic_cdk::api::time(), session_id)
}

/// Simple single-chunk upload for small files
#[update]
fn upload_file(
    filename: String,
    content_type: String,
    media_type: MediaType,
    data: Vec<u8>,
) -> Result<String, String> {
    service::upload_file_impl(ic_cdk::caller(), ic_cdk::api::time(), filename, content_type, media_type, data)
}

/// Cancel an upload session
#[update]
fn cancel_upload(session_id: String) -> Result<(), String> {
    service::cancel_upload_impl(ic_cdk::caller(), session_id)
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
