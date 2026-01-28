use candid::Principal;
use crate::types::*;
use crate::state::*;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Stable deterministic hash for answer verification (djb2 variant)
pub fn stable_hash(data: &[u8]) -> [u8; 32] {
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
pub fn rebuild_quiz_index() {
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
pub fn increment_version() -> u64 {
    CONTENT_VERSION.with(|v| {
        let mut cell = v.borrow_mut();
        let current = *cell.get();
        let new_version = current + 1;
        cell.set(new_version).expect("Failed to increment version");
        new_version
    })
}

/// Get content version for a specific node
pub fn get_content_version(content_id: &str) -> u64 {
    CONTENT_VERSIONS.with(|v| v.borrow().get(&content_id.to_string()).unwrap_or(0))
}

/// Set content version for a specific node
pub fn set_content_version(content_id: &str, version: u64) {
    CONTENT_VERSIONS.with(|v| {
        v.borrow_mut().insert(content_id.to_string(), version);
    });
}

/// Convert internal ContentNode to public version (without answers)
pub fn to_public_node(node: &ContentNode) -> PublicContentNode {
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
pub fn add_content_node_internal(node: ContentNode, proposal_id: Option<u64>) -> Result<(), String> {
    let id = node.id.clone();
    let now = ic_cdk::api::time();
    
    // Check if this is an update
    let is_update = CONTENT_NODES.with(|c| c.borrow().contains_key(&id));
    
    // If updating, save snapshot for version history
    if is_update {
        CONTENT_NODES.with(|c| {
            if let Some(old_node) = c.borrow().get(&id) {
                let current_version = get_content_version(&id);
                VERSION_HISTORY.with(|h| {
                    h.borrow_mut().insert(
                        VersionKey { content_id: id.clone(), version: current_version },
                        ContentSnapshot {
                            content: old_node.clone(),
                            modified_at: now,
                            modified_by_proposal: proposal_id.unwrap_or(0),
                            change_type: ChangeType::Updated,
                        }
                    );
                });
            }
        });
    } else {
        // New node - save creation snapshot
        VERSION_HISTORY.with(|h| {
            h.borrow_mut().insert(
                VersionKey { content_id: id.clone(), version: 1 },
                ContentSnapshot {
                    content: node.clone(),
                    modified_at: now,
                    modified_by_proposal: proposal_id.unwrap_or(0),
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

// ... Additional functions related to content loading and version history can be added here
// or kept in lib.rs if they are simple wrappers.
// But mostly the heavy logic for loading should ideally be here.
// For now, I've moved `add_content_node_internal`.

/// Resume incomplete jobs after upgrade
pub async fn resume_incomplete_jobs() {
    let jobs: Vec<LoadingJob> = LOADING_JOBS.with(|j| {
        j.borrow().iter()
            .map(|(_, job)| job)
            .filter(|job| job.status == LoadingStatus::InProgress)
            .collect()
    });
    
    for job in jobs {
        ic_cdk::println!("Resuming loading job for proposal {}", job.proposal_id);
        // We can't easily call resume_loading here without recursion and self-call setup which is tricky.
        // For now, assume a simple print or partial logic.
        // In a real refactor, `continue_loading` logic should be in service or separated.
        // Call the canister's continue_loading method (which calls continue_loading_impl)
        let self_id = ic_cdk::api::id();
        let _ = ic_cdk::call::<_, ()>(self_id, "continue_loading", (job.proposal_id,)).await;
    }
}

/// Start loading content from staging (called by governance on proposal approval)
pub async fn start_content_load_impl(
    proposal_id: u64,
    staging_canister: Principal,
    staging_path: String,
    content_hash: String,
    total_units: u32,
) -> Result<(), String> {
    // Check if job already exists
    let exists = LOADING_JOBS.with(|jobs| jobs.borrow().contains_key(&proposal_id));
    if exists {
        return Err(format!("Loading job for proposal {} already exists", proposal_id));
    }
    
    // Create new job
    let job = LoadingJob {
        proposal_id,
        staging_canister,
        staging_path,
        content_hash,
        total_units,
        loaded_units: 0,
        status: LoadingStatus::InProgress,
        last_error: None,
        started_at: ic_cdk::api::time(),
        updated_at: ic_cdk::api::time(),
    };
    
    LOADING_JOBS.with(|jobs| jobs.borrow_mut().insert(proposal_id, job));
    
    // Start loading process (kick off the first batch)
    // We use a self-call to `continue_loading` to ensure atomicity of the start command
    // and to run the heavy lifting in a separate message execution
    let self_id = ic_cdk::api::id();
    let _ = ic_cdk::call::<_, ()>(self_id, "continue_loading", (proposal_id,)).await;
    
    Ok(())
}

/// Continue loading (self-call pattern for resilience)
pub async fn continue_loading_impl(proposal_id: u64) -> Result<(), String> {
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

/// Delete a content node
pub fn delete_content_node_impl(id: String, proposal_id: u64) -> Result<(), String> {
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

/// Verify quiz answers implementation
pub fn verify_quiz_impl(content_id: String, answers: Vec<u8>) -> (bool, u64, u64) {
    let quiz = match QUIZ_INDEX.with(|q| q.borrow().get(&content_id)) {
        Some(q) => q,
        None => return (false, 0, 0),
    };

    let total = quiz.question_count as u64;
    if total == 0 || total != answers.len() as u64 {
        return (false, 0, total);
    }

    let mut correct = 0;
    for (i, ans) in answers.iter().enumerate() {
        if stable_hash(&ans.to_le_bytes()) == quiz.answer_hashes[i] {
            correct += 1;
        }
    }

    // Default pass threshold if not specified/synced
    // In the new architecture, Shards verify locally using their own config.
    // This remote verification is a fallback.
    let pass_threshold = 80; 
    let passed = (correct * 100 / total) >= pass_threshold;

    (passed, correct, total)
}
