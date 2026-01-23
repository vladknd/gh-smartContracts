# Content Governance

**Last Updated**: January 2026  
**Status**: Implemented  
**Priority**: Medium - Enables democratic content management

---

## Table of Contents
1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Complete Content Proposal Flow](#complete-content-proposal-flow)
4. [Media Assets Canister](#media-assets-canister)
5. [Staging Assets Canister](#staging-assets-canister)
6. [Content Proposal Types](#content-proposal-types)
7. [Automatic Content Loading](#automatic-content-loading)
8. [Version History & Audit Trail](#version-history--audit-trail)
9. [Deployment & Configuration](#deployment--configuration)

---

## Overview

Content modifications in the learning engine go through a governance proposal system, ensuring:
- **Democratic control** - Board members vote on content changes
- **Transparency** - All changes are tracked and auditable
- **Rollback capability** - Full version history maintained
- **Quality control** - Review process before content goes live
- **Automatic loading** - Content loads into learning engine automatically after approval

### Key Components

| Component | Role |
|-----------|------|
| `media_assets` | **Permanent** storage for videos/audio/images/PDFs |
| `staging_assets` | **Temporary** storage for content awaiting approval |
| `governance_canister` | Manages proposals and voting |
| `learning_engine` | Final destination for content (ContentNodes) |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         CONTENT GOVERNANCE ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│                              ┌─────────────────────┐                             │
│                              │  governance_canister │                            │
│                              │  ─────────────────── │                            │
│                              │  • Create proposals  │                            │
│                              │  • Vote on content   │                            │
│                              │  • Execute approved  │                            │
│                              └──────────┬──────────┘                             │
│                                         │                                        │
│                          execute_proposal()                                      │
│                  ┌──────────────┴──────────────┐                                │
│                  │                             │                                 │
│                  ▼                             ▼                                 │
│    ┌─────────────────────┐       ┌─────────────────────┐                        │
│    │   staging_assets    │       │   learning_engine   │                        │
│    │   ───────────────   │──────>│   ───────────────   │                        │
│    │   • Stage content   │ load  │   • Store nodes     │                        │
│    │   • Track status    │       │   • Quiz index      │                        │
│    │   • Provide chunks  │       │   • Version history │                        │
│    └─────────────────────┘       └─────────────────────┘                        │
│              ▲                                                                   │
│              │ stage_content()                                                   │
│              │                                                                   │
│    ┌─────────┴───────────┐                                                      │
│    │      Creator        │                                                       │
│    │      ───────        │                                                       │
│    │   • Upload media    │────────┐                                             │
│    │   • Stage content   │        │                                              │
│    │   • Create proposal │        │ upload_file()                                │
│    └─────────────────────┘        │                                              │
│                                   ▼                                              │
│                      ┌─────────────────────┐                                    │
│                      │    media_assets     │                                     │
│                      │    ────────────     │                                     │
│                      │    • Permanent      │                                     │
│                      │    • Immutable      │                                     │
│                      │    • Deduplicated   │                                     │
│                      └─────────────────────┘                                    │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Complete Content Proposal Flow

This is the end-to-end flow for uploading content and getting it approved through governance:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        COMPLETE CONTENT PROPOSAL FLOW                           │
└─────────────────────────────────────────────────────────────────────────────────┘

STEP 1: UPLOAD MEDIA FILES
══════════════════════════
   Creator                          media_assets
      │                                  │
      │  upload_file(video.mp4, ...)     │
      │─────────────────────────────────>│
      │                                  │ (stores file, computes SHA256)
      │            file_hash             │
      │<─────────────────────────────────│
      │                                  │

STEP 2: PREPARE & STAGE CONTENT
═══════════════════════════════
   Creator                          staging_assets
      │                                  │
      │  (builds ContentNode array       │
      │   with media URLs from Step 1)   │
      │                                  │
      │  stage_content(title, desc,      │
      │                nodes[])          │
      │─────────────────────────────────>│
      │                                  │ (stores nodes, computes hash)
      │         content_hash             │
      │<─────────────────────────────────│
      │                                  │

STEP 3: CREATE GOVERNANCE PROPOSAL
══════════════════════════════════
   Creator                          governance_canister
      │                                  │
      │  create_add_content_proposal({   │
      │    staging_canister,             │
      │    content_hash,                 │
      │    unit_count,                   │
      │    title, description            │
      │  })                              │
      │─────────────────────────────────>│
      │                                  │ (creates proposal)
      │         proposal_id              │
      │<─────────────────────────────────│
      │                                  │

STEP 4: LINK PROPOSAL TO STAGED CONTENT
═══════════════════════════════════════
   Creator                          staging_assets
      │                                  │
      │  set_proposal_id(content_hash,   │
      │                  proposal_id)    │
      │─────────────────────────────────>│
      │              OK                  │
      │<─────────────────────────────────│
      │                                  │

STEP 5: BOARD VOTES
═══════════════════
   Board Members                    governance_canister
      │                                  │
      │  vote(proposal_id, true/false)   │
      │─────────────────────────────────>│
      │              OK                  │
      │<─────────────────────────────────│
      │     (repeat for each member)     │

STEP 6: FINALIZE & EXECUTE
══════════════════════════
   Anyone                           governance_canister
      │                                  │
      │  finalize_proposal(proposal_id)  │
      │─────────────────────────────────>│
      │          "Approved"              │
      │<─────────────────────────────────│
      │                                  │
      │  execute_proposal(proposal_id)   │
      │─────────────────────────────────>│
      │                                  │
      └──────────────────────────────────┘

              governance_canister               staging_assets
                     │                               │
                     │  mark_loading(content_hash)   │
                     │──────────────────────────────>│
                     │              OK               │
                     │<──────────────────────────────│
                     │                               │

              governance_canister               learning_engine
                     │                               │
                     │  start_content_load(          │
                     │    proposal_id,               │
                     │    staging_canister,          │
                     │    content_hash,              │
                     │    unit_count                 │
                     │  )                            │
                     │──────────────────────────────>│
                     │              OK               │
                     │<──────────────────────────────│
                     │                               │

STEP 7: LEARNING ENGINE LOADS CONTENT (RESILIENT)
══════════════════════════════════════════════════
              learning_engine                   staging_assets
                     │                               │
        ┌───────────>│                               │
        │            │  get_content_chunk(           │
        │            │    content_hash,              │
        │            │    offset, limit              │
        │            │  )                            │
        │            │──────────────────────────────>│
        │            │       ContentNode[]           │
        │            │<──────────────────────────────│
        │            │                               │
  LOOP  │            │  (adds nodes to               │
  until │            │   CONTENT_NODES storage)      │
  done  │            │                               │
        │            │  self-call: continue_loading  │
        └────────────│                               │
                     │                               │
              (when complete)                        │
                     │                               │
                     │  mark_loaded(content_hash)    │
                     │──────────────────────────────>│
                     │              OK               │
                     │<──────────────────────────────│
                     │                               │

STEP 8: CLEANUP (OPTIONAL)
══════════════════════════
   Creator                          staging_assets
      │                                  │
      │  delete_staged_content(          │
      │    content_hash                  │
      │  )                               │
      │─────────────────────────────────>│
      │              OK                  │
      │<─────────────────────────────────│
      │                                  │

┌─────────────────────────────────────────────────────────────────────────────────┐
│                              CONTENT NOW LIVE!                                  │
│                                                                                 │
│  • ContentNodes stored in learning_engine                                       │
│  • Quiz index populated for O(1) lookup                                         │
│  • Version history recorded for audit                                           │
│  • Media files permanently stored in media_assets                               │
│  • Staging content can be deleted to free space                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Media Assets Canister

The `media_assets` canister provides **permanent, immutable storage** for media files.

### Features

| Feature | Description |
|---------|-------------|
| **Chunked Upload** | Large files (up to 100MB) uploaded in 2MB chunks |
| **Content-Addressed** | SHA256 hash as key - automatic deduplication |
| **Immutable** | Files cannot be deleted or modified once uploaded |
| **Access Control** | Configurable allowed uploaders list |

### Key Functions

```rust
// Simple upload for small files (< 2MB)
upload_file(filename, content_type, media_type, data) -> Result<file_hash>

// Chunked upload for large files
start_upload(filename, content_type, media_type, expected_size) -> Result<session_id>
upload_chunk(session_id, chunk_index, data) -> Result<()>
finalize_upload(session_id) -> Result<file_hash>

// Retrieval
get_file_metadata(hash) -> Option<FileMetadata>
get_file_chunk(hash, chunk_index) -> Option<blob>
get_file(hash) -> Result<blob>  // Small files only
get_file_url(hash) -> Result<url>
```

### Media Types

```rust
enum MediaType {
    Video,   // .mp4, .webm, etc.
    Audio,   // .mp3, .wav, etc.
    Image,   // .png, .jpg, .webp, etc.
    PDF,     // .pdf
    Other,   // Any other file type
}
```

### Example: Uploading a Video

```typescript
// TypeScript frontend example
import { media_assets } from "./declarations/media_assets";

// For small files (< 2MB)
const smallImageHash = await media_assets.upload_file(
    "lesson-cover.png",
    "image/png",
    { Image: null },
    imageData
);

// For large files (chunked)
const sessionId = await media_assets.start_upload(
    "lesson-video.mp4",
    "video/mp4",
    { Video: null },
    videoData.length
);

const chunkSize = 2 * 1024 * 1024; // 2MB
for (let i = 0; i < Math.ceil(videoData.length / chunkSize); i++) {
    const chunk = videoData.slice(i * chunkSize, (i + 1) * chunkSize);
    await media_assets.upload_chunk(sessionId, i, chunk);
}

const videoHash = await media_assets.finalize_upload(sessionId);

// Get the URL for use in ContentNode
const videoUrl = await media_assets.get_file_url(videoHash);
// => "https://xxxx-xxx.raw.icp0.io/file/abc123..."
```

---

## Staging Assets Canister

The `staging_assets` canister provides **temporary storage** for content awaiting governance approval.

### Staging Status Flow

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           STAGING STATUS LIFECYCLE                              │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   ┌─────────┐      ┌─────────────────┐      ┌─────────┐      ┌────────┐        │
│   │ Pending │ ───> │ ProposalCreated │ ───> │ Loading │ ───> │ Loaded │        │
│   └─────────┘      └─────────────────┘      └─────────┘      └────────┘        │
│        │                    │                                     │             │
│        │                    │                                     │             │
│        │                    ▼                                     ▼             │
│        │              ┌──────────┐                          (can delete)        │
│        │              │ Rejected │                                              │
│        │              └──────────┘                                              │
│        │                    │                                                   │
│        └────────────────────┴──────────> (can delete)                          │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Key Functions

```rust
// Stage content (returns content_hash)
stage_content(title, description, nodes: Vec<ContentNode>) -> Result<content_hash>

// Link to proposal
set_proposal_id(content_hash, proposal_id) -> Result<()>

// Status updates (called by governance/learning_engine)
mark_loading(content_hash) -> Result<()>
mark_loaded(content_hash) -> Result<()>
mark_rejected(content_hash) -> Result<()>

// Content retrieval (for learning_engine)
get_content_chunk(content_hash, offset, limit) -> Vec<ContentNode>
get_all_content_nodes(content_hash) -> Result<Vec<ContentNode>>

// Cleanup
delete_staged_content(content_hash) -> Result<()>
```

### Example: Staging Content

```typescript
// TypeScript frontend example
import { staging_assets } from "./declarations/staging_assets";

// Build ContentNode array
const contentNodes = [
    {
        id: "book:1",
        parent_id: [],
        order: 1,
        display_type: "Book",
        title: "Environmental Science 101",
        description: ["Introduction to environmental science"],
        content: [],
        paraphrase: [],
        media: [],
        quiz: [],
        created_at: BigInt(Date.now() * 1000000),
        updated_at: BigInt(Date.now() * 1000000),
        version: 1n,
    },
    {
        id: "book:1:ch:1",
        parent_id: ["book:1"],
        order: 1,
        display_type: "Chapter",
        title: "Climate Basics",
        // ... etc
    },
    // ... more nodes
];

// Stage the content
const contentHash = await staging_assets.stage_content(
    "Environmental Science 101",
    "Complete introductory course on environmental science",
    contentNodes
);

console.log("Staged content hash:", contentHash);
// => "8f7e6d5c4b3a2190..."
```

---

## Content Proposal Types

### AddContentFromStaging

Add new content from staging canister.

```rust
AddContentFromStagingPayload {
    staging_canister: Principal,   // ID of staging_assets canister
    staging_path: String,          // Content hash in staging
    content_hash: String,          // SHA256 of content
    content_title: String,         // Human-readable title
    unit_count: u32,               // Number of ContentNodes
}
```

### UpdateGlobalQuizConfig

Update global quiz settings (applies to ALL quizzes).

```rust
UpdateGlobalQuizConfigPayload {
    new_reward_amount: Option<u64>,     // Tokens for completing any quiz
    new_pass_threshold: Option<u8>,     // e.g., 60%
    new_max_attempts: Option<u8>,       // Daily attempts per quiz
}
```

### DeleteContentNode

Remove content (with audit trail).

```rust
DeleteContentNodePayload {
    content_id: String,   // ID of node to delete
    reason: String,       // Reason for deletion
}
```

---

## Automatic Content Loading

The learning engine uses a **resilient self-call pattern** for loading content:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        RESILIENT LOADING PATTERN                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   1. Governance calls start_content_load()                                       │
│                                                                                  │
│   2. Learning engine creates LoadingJob in STABLE storage                        │
│      (survives upgrades!)                                                        │
│                                                                                  │
│   3. Fetches batch of 10 ContentNodes from staging                              │
│                                                                                  │
│   4. Processes each node:                                                        │
│      - Adds to CONTENT_NODES                                                     │
│      - Updates CHILDREN_INDEX                                                    │
│      - Updates QUIZ_INDEX if has quiz                                            │
│      - Records VERSION_HISTORY                                                   │
│                                                                                  │
│   5. If more nodes remaining:                                                    │
│      - Self-calls continue_loading()                                             │
│      - This allows other messages to be processed (prevents blocking)            │
│                                                                                  │
│   6. If error occurs:                                                            │
│      - Sets status to Paused                                                     │
│      - Saves error message                                                       │
│      - Can be resumed later with resume_loading()                                │
│                                                                                  │
│   7. On completion:                                                              │
│      - Calls staging.mark_loaded()                                               │
│      - Sets job status to Completed                                              │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### LoadingJob Structure

```rust
struct LoadingJob {
    proposal_id: u64,
    staging_canister: Principal,
    staging_path: String,
    content_hash: String,
    total_units: u32,
    loaded_units: u32,      // Progress tracking
    status: LoadingStatus,  // InProgress | Completed | Failed | Paused
    last_error: Option<String>,
    started_at: u64,
    updated_at: u64,
}
```

---

## Version History & Audit Trail

Every content modification is tracked for full auditability:

```rust
struct ContentSnapshot {
    content: ContentNode,          // Copy of the node
    modified_at: u64,              // Timestamp
    modified_by_proposal: u64,     // Which proposal made the change
    change_type: ChangeType,       // Created | Updated | Deleted
}
```

### Query Functions

```rust
// Get all versions of a content node
get_content_version_history(content_id) -> Vec<(version, ContentSnapshot)>

// Get content at a specific version
get_content_at_version(content_id, version) -> Option<ContentNode>

// Get all changes made by a proposal
get_changes_by_proposal(proposal_id) -> Vec<(content_id, ChangeType)>
```

---

## Deployment & Configuration

### Deploy Assets Canisters

```bash
# Deploy media_assets (permanent storage)
dfx deploy media_assets --argument '(record { 
    allowed_uploaders = vec {}  
    // Empty = anyone can upload
    // Or specify principals: vec { principal "xxx-xxx" }
})'

# Deploy staging_assets (temporary storage)
dfx deploy staging_assets --argument '(record { 
    governance_canister_id = principal "GOVERNANCE_CANISTER_ID"; 
    learning_engine_id = principal "LEARNING_ENGINE_ID"
})'
```

### Configure Governance Canister

```bash
# Set learning engine ID in governance canister
dfx canister call governance_canister set_learning_engine_id \
    '(principal "LEARNING_ENGINE_ID")'
```

### Configure Learning Engine

```bash
# Set governance canister ID in learning engine (during deploy)
dfx deploy learning_engine --argument '(record { 
    staking_hub_id = principal "STAKING_HUB_ID";
    governance_canister_id = opt principal "GOVERNANCE_CANISTER_ID"
})'
```

---

## Summary

| Canister | Purpose | Storage Type |
|----------|---------|--------------|
| `media_assets` | Videos, images, audio, PDFs | Permanent, immutable |
| `staging_assets` | ContentNodes before approval | Temporary, deletable |
| `governance_canister` | Proposal management | Persistent |
| `learning_engine` | Final content storage | Persistent |

**Key Features:**
- ✅ Content-addressed media storage with deduplication
- ✅ Chunked uploads for large files
- ✅ Democratic governance for all content changes
- ✅ Resilient content loading (survives upgrades)
- ✅ Full version history for audit trails
- ✅ Automatic cleanup of staging content
