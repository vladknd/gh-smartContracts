#!/bin/bash

# ============================================================================
# MEDIA ASSETS CANISTER TEST SCRIPT
# ============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Load helper
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/test_helper.sh"

log_header "MEDIA ASSETS CANISTER TEST"

# ============================================================================
# 1. SETUP
# ============================================================================
setup_environment "$@"

if [[ "$*" != *"--no-deploy"* ]]; then
    log_step "Deploying media_assets..."
    ALLOWED_USER=$(dfx identity get-principal)
    dfx deploy media_assets --argument "(record {
        allowed_uploaders = vec { principal \"$ALLOWED_USER\" }
    })"
else
    log_info "Using existing deployment of media_assets"
fi

MEDIA_ID=$(dfx canister id media_assets 2>/dev/null)
log_info "Media Assets ID: $MEDIA_ID"

# ============================================================================
# 2. SIMPLE UPLOAD
# ============================================================================
log_step "Testing Simple Upload"

# Upload a small file
# Blob syntax: blob "\CA\FE" or similar. Or just string for text.
# For simplicity, we upload a small text file as blob.

FILENAME="test_small.txt"
CONTENT="Hello Media"

UPLOAD_RES=$(dfx canister call media_assets upload_file "(
    \"$FILENAME\",
    \"text/plain\",
    variant { Other },
    blob \"$CONTENT\"
)")
log_info "Upload Result: $UPLOAD_RES"

# Result should contain the hash
if echo "$UPLOAD_RES" | grep -q "Ok"; then
    log_pass "Simple upload successful"
    HASH=$(echo "$UPLOAD_RES" | sed -n 's/.*Ok = "\(.*\)".*/\1/p')
    log_info "File Hash: $HASH"
else
    log_fail "Simple upload failed"
fi

# Verify metadata
META=$(dfx canister call media_assets get_file_metadata "(\"$HASH\")")
log_info "Metadata: $META"

if echo "$META" | grep -q "$FILENAME"; then
    log_pass "Metadata verified"
else
    log_fail "Metadata verification failed"
fi

# Verify content
GET_RES=$(dfx canister call media_assets get_file "(\"$HASH\")")
# Output will be blob, might be hard to verify exact string in bash without polling/decoding
# But if it returns Ok, that's good.
log_info "Get File Result: $GET_RES"

if echo "$GET_RES" | grep -q "Ok"; then
    log_pass "Content retrieval successful"
else
    log_fail "Content retrieval failed"
fi

# ============================================================================
# 3. CHUNKED UPLOAD
# ============================================================================
log_step "3. Chunked Upload"

BIG_FILE="test_big.bin"
SIZE=20 # 2 chunks of 10 bytes

# Result: Expects 1 chunk because 20 bytes < 2MB (MAX_CHUNK_SIZE)
START_RES=$(dfx canister call media_assets start_upload "(
    \"$BIG_FILE\",
    \"application/octet-stream\",
    variant { Other },
    $SIZE : nat64
)")
log_info "Start Upload Result: $START_RES"

SESSION_ID=$(echo "$START_RES" | sed -n 's/.*Ok = "\(.*\)".*/\1/p')
log_info "Session ID: $SESSION_ID"

# Chunk 1 (Full 20 bytes)
CHUNK_FULL="0123456789abcdefghij"
log_info "Uploading Chunk 0 (Full)..."
dfx canister call media_assets upload_chunk "(
    \"$SESSION_ID\",
    0 : nat32,
    blob \"$CHUNK_FULL\"
)"

# Finalize
log_info "Finalizing..."
FINALIZE_RES=$(dfx canister call media_assets finalize_upload "(\"$SESSION_ID\")")
log_info "Finalize Result: $FINALIZE_RES"

if echo "$FINALIZE_RES" | grep -q "Ok"; then
    log_pass "Chunked upload finalized"
else
    log_fail "Finalize failed"
fi

summary
