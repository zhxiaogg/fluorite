#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "Fluorite E2E Interoperability Test"
echo "========================================="
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/fixtures"
RUST_TO_TS_DIR="$FIXTURES_DIR/rust_to_ts"
TS_TO_RUST_DIR="$FIXTURES_DIR/ts_to_rust"

# Create fixture directories
mkdir -p "$RUST_TO_TS_DIR"
mkdir -p "$TS_TO_RUST_DIR"

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

info() {
    echo -e "${YELLOW}→ INFO${NC}: $1"
}

# Step 1: Build Rust demo
echo "Step 1: Building Rust demo..."
cd "$SCRIPT_DIR/../../examples/demo"
if cargo build --quiet 2>/dev/null; then
    pass "Rust demo built successfully"
else
    fail "Rust demo build failed"
    exit 1
fi

# Step 2: Build Fluorite CLI for TypeScript generation
echo ""
echo "Step 2: Building Fluorite CLI..."
cd "$SCRIPT_DIR/../../"
if cargo build --package fluorite_codegen --bin fluorite --quiet 2>/dev/null; then
    pass "Fluorite CLI built successfully"
else
    fail "Fluorite CLI build failed"
    exit 1
fi

# Step 3: Generate TypeScript types from .fl files
echo ""
echo "Step 3: Generating TypeScript types..."
cd "$SCRIPT_DIR/../../examples/demo-ts"
if "$SCRIPT_DIR/../../target/debug/fluorite" ts --inputs ../demo/fluorite/common.fl ../demo/fluorite/demo.fl --output ./generated 2>/dev/null; then
    pass "TypeScript types generated successfully"
else
    fail "TypeScript type generation failed"
    exit 1
fi

# Step 4: Install TypeScript dependencies and build
echo ""
echo "Step 4: Installing TypeScript dependencies..."
if [ ! -d "node_modules" ]; then
    npm install --silent 2>/dev/null || true
fi

# Build TypeScript
info "Building TypeScript..."
npx tsc 2>/dev/null || true

# Check if dist/index.js exists
if [ -f "dist/src/index.js" ]; then
    pass "TypeScript built successfully"
else
    info "Using ts-node for direct execution"
fi

# Step 5: Test Rust → TypeScript
echo ""
echo "========================================="
echo "Step 5: Testing Rust → TypeScript"
echo "========================================="

# Rust writes JSON files
cd "$SCRIPT_DIR/../../examples/demo"
info "Running Rust demo to write JSON files..."
cargo run --quiet -- write --output "$RUST_TO_TS_DIR" 2>/dev/null

# Verify Rust wrote files
REQUIRED_FILES=(
    "user_male.json"
    "user_female.json"
    "order.json"
    "event_user_created.json"
    "event_order_placed.json"
    "event_message.json"
    "event_ping.json"
    "address.json"
)

ALL_EXIST=true
for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$RUST_TO_TS_DIR/$file" ]; then
        ALL_EXIST=false
        fail "Rust failed to write $file"
    fi
done

if [ "$ALL_EXIST" = true ]; then
    pass "Rust wrote all JSON files successfully"
fi

# TypeScript reads and validates Rust's JSON
cd "$SCRIPT_DIR/../../examples/demo-ts"
info "Running TypeScript to read Rust's JSON files..."
if node dist/src/index.js --read "$RUST_TO_TS_DIR" 2>/dev/null; then
    pass "TypeScript successfully read Rust's JSON files"
else
    if npx ts-node src/index.ts --read "$RUST_TO_TS_DIR" 2>/dev/null; then
        pass "TypeScript successfully read Rust's JSON files (ts-node)"
    else
        fail "TypeScript failed to read Rust's JSON files"
    fi
fi

# Step 6: Test TypeScript → Rust
echo ""
echo "========================================="
echo "Step 6: Testing TypeScript → Rust"
echo "========================================="

# TypeScript writes JSON files
cd "$SCRIPT_DIR/../../examples/demo-ts"
info "Running TypeScript to write JSON files..."
if node dist/src/index.js --write "$TS_TO_RUST_DIR" 2>/dev/null; then
    pass "TypeScript wrote JSON files successfully"
else
    if npx ts-node src/index.ts --write "$TS_TO_RUST_DIR" 2>/dev/null; then
        pass "TypeScript wrote JSON files successfully (ts-node)"
    else
        fail "TypeScript failed to write JSON files"
    fi
fi

# Verify TypeScript wrote files
ALL_EXIST=true
for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$TS_TO_RUST_DIR/$file" ]; then
        ALL_EXIST=false
        fail "TypeScript failed to write $file"
    fi
done

if [ "$ALL_EXIST" = true ]; then
    pass "TypeScript wrote all JSON files successfully"
fi

# Rust reads and validates TypeScript's JSON
cd "$SCRIPT_DIR/../../examples/demo"
info "Running Rust demo to read TypeScript's JSON files..."
if cargo run --quiet -- read --input "$TS_TO_RUST_DIR" 2>/dev/null; then
    pass "Rust successfully read TypeScript's JSON files"
else
    fail "Rust failed to read TypeScript's JSON files"
fi

# Step 7: Summary
echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All interoperability tests passed!${NC}"
    echo ""
    echo "Tested scenarios:"
    echo "  ✓ Rust serializes → TypeScript deserializes"
    echo "  ✓ TypeScript serializes → Rust deserializes"
    echo "  ✓ User objects with UUID, DateTime, Optional fields"
    echo "  ✓ Order with nested Address, Vec<OrderItem>, Decimal"
    echo "  ✓ Gender enum (Male/Female/Other)"
    echo "  ✓ Status enum (Active/Inactive/Suspended)"
    echo "  ✓ DemoEvent union (UserCreated/OrderPlaced/Message/Ping)"
    echo "  ✓ Cross-package imports (common → demo)"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
