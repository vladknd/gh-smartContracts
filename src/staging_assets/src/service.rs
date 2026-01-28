use candid::{Principal, Encode};
use sha2::{Sha256, Digest};
use crate::state::*;
use crate::types::*;

pub mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn is_allowed_stager(caller: &Principal) -> bool {
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

pub fn is_governance_canister(caller: &Principal) -> bool {
    GOVERNANCE_CANISTER_ID.with(|id| *id.borrow().get() == *caller)
}

pub fn is_learning_engine(caller: &Principal) -> bool {
    LEARNING_ENGINE_ID.with(|id| *id.borrow().get() == *caller)
}

pub fn stage_content_impl(
    caller: Principal,
    title: String,
    description: String,
    nodes: Vec<ContentNode>,
) -> Result<String, String> {
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

pub fn set_proposal_id_impl(caller: Principal, content_hash: String, proposal_id: u64) -> Result<(), String> {
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

pub fn mark_loading_impl(caller: Principal, content_hash: String) -> Result<(), String> {
    if !is_governance_canister(&caller) && !ic_cdk::api::is_controller(&caller) {
        return Err("Only governance canister can mark as loading".to_string());
    }
    
    let mut content = STAGED_CONTENT.with(|s| s.borrow().get(&content_hash))
        .ok_or("Staged content not found")?;
    
    content.status = StagingStatus::Loading;
    
    STAGED_CONTENT.with(|s| s.borrow_mut().insert(content_hash, content));
    
    Ok(())
}

pub fn mark_loaded_impl(caller: Principal, content_hash: String) -> Result<(), String> {
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

pub fn mark_rejected_impl(caller: Principal, content_hash: String) -> Result<(), String> {
    if !is_governance_canister(&caller) && !ic_cdk::api::is_controller(&caller) {
        return Err("Only governance canister can mark as rejected".to_string());
    }
    
    let mut content = STAGED_CONTENT.with(|s| s.borrow().get(&content_hash))
        .ok_or("Staged content not found")?;
    
    content.status = StagingStatus::Rejected;
    
    STAGED_CONTENT.with(|s| s.borrow_mut().insert(content_hash, content));
    
    Ok(())
}

pub fn delete_staged_content_impl(caller: Principal, content_hash: String) -> Result<(), String> {
    let content = STAGED_CONTENT.with(|s| s.borrow().get(&content_hash))
        .ok_or("Staged content not found")?;
    
    if content.stager != caller 
        && !is_governance_canister(&caller) 
        && !is_learning_engine(&caller)
        && !ic_cdk::api::is_controller(&caller) {
        return Err("Not authorized to delete".to_string());
    }
    
    match content.status {
        StagingStatus::Loaded | StagingStatus::Rejected => {
            STAGED_CONTENT.with(|s| s.borrow_mut().remove(&content_hash));
            Ok(())
        }
        StagingStatus::Pending => {
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
