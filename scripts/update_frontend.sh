#!/bin/bash

# ============================================================================
# Update Frontend Script
# Copies TypeScript declarations and configuration to the frontend project
# ============================================================================

# Destination Directories
DASHBOARD_ROOT="/mnt/c/LIB/CODE/gh-dashboard"
DECLARATIONS_DIR="$DASHBOARD_ROOT/declarations"
CONFIG_FILE="$DASHBOARD_ROOT/ic.config.json"

# Source declarations directory
SRC_DECLARATIONS="src/declarations"

# Ensure directories exist
mkdir -p "$DECLARATIONS_DIR"

echo "============================================"
echo "Updating Frontend Declarations"
echo "============================================"

echo ""
echo "Copying TypeScript declarations..."

# Automatically discover all canisters in the declarations folder
COPIED_COUNT=0
MISSING_COUNT=0

for canister_dir in "$SRC_DECLARATIONS"/*/; do
    if [ -d "$canister_dir" ]; then
        # Get canister name from directory path
        canister=$(basename "$canister_dir")
        
        SRC_DIR="$SRC_DECLARATIONS/$canister"
        DEST_DIR="$DECLARATIONS_DIR/$canister"
        
        # Create destination directory
        mkdir -p "$DEST_DIR"
        
        # Copy all TypeScript-related files
        # .did.d.ts - TypeScript type definitions
        # .did.js   - JavaScript bindings with IDL
        # index.js  - Main export file
        # index.d.ts - Type definitions for index
        # .did      - Candid interface (useful for reference)
        
        cp "$SRC_DIR"/*.d.ts "$DEST_DIR/" 2>/dev/null
        cp "$SRC_DIR"/*.js "$DEST_DIR/" 2>/dev/null
        cp "$SRC_DIR"/*.did "$DEST_DIR/" 2>/dev/null
        
        # Check if files were actually copied
        if ls "$DEST_DIR"/*.js 1> /dev/null 2>&1; then
            echo "  ✅ $canister"
            ((COPIED_COUNT++))
        else
            echo "  ⚠️  $canister (no files copied)"
            ((MISSING_COUNT++))
        fi
    fi
done

echo ""
echo "Copied $COPIED_COUNT canisters"
if [ $MISSING_COUNT -gt 0 ]; then
    echo "⚠️  $MISSING_COUNT canisters had issues"
fi

echo ""
echo "Copying IC configuration..."

# Copy the entire ic.config.json file
# This includes network, host, all canister IDs, and founder addresses
if [ -f "ic.config.json" ]; then
    cp ic.config.json "$CONFIG_FILE"
    echo "  ✅ ic.config.json"
else
    echo "  ⚠️  ic.config.json not found"
fi

echo ""
echo "Copying integration documentation..."

# Create docs directory if it doesn't exist
DOCS_DIR="$DASHBOARD_ROOT/docs"
mkdir -p "$DOCS_DIR"

# Copy all markdown documentation
for doc in docs/*.md; do
    if [ -f "$doc" ]; then
        filename=$(basename "$doc")
        cp "$doc" "$DOCS_DIR/"
        echo "  ✅ $filename"
    fi
done

echo ""
echo "============================================"
echo "Summary"
echo "============================================"
echo "Declarations copied to: $DECLARATIONS_DIR"
echo "  - $COPIED_COUNT canisters"
echo "Config copied to: $CONFIG_FILE"
echo "Docs copied to: $DOCS_DIR"
echo ""
echo "Done! 🎉"
