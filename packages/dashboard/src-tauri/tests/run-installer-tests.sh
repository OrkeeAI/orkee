#!/usr/bin/env bash
# ABOUTME: Test runner for installer scripts
# ABOUTME: Validates script structure, error handling, and documentation

set -euo pipefail

TESTS_PASSED=0
TESTS_FAILED=0
TEST_DIR="$(cd "$(dirname "$0")" && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

pass() { echo -e "${GREEN}✓${NC} $1"; ((TESTS_PASSED++)); }
fail() { echo -e "${RED}✗${NC} $1: $2"; ((TESTS_FAILED++)); }
test_file() {
    local file="$1"
    local name="$(basename "$file")"
    if [ ! -f "$file" ]; then
        fail "$name" "not found"
        return 1
    fi
    if grep -q 'set -euo pipefail' "$file"; then
        pass "$name: strict mode"
    else
        fail "$name" "no strict mode"
    fi
    if grep -q 'ABOUTME' "$file"; then
        pass "$name: documented"
    else
        fail "$name" "no ABOUTME"
    fi
    return 0
}

echo "Installer Scripts Test Suite"
echo "=============================="
echo ""

# Test prepare-binaries.sh
echo "Testing prepare-binaries.sh..."
SCRIPT="$TEST_DIR/../../prepare-binaries.sh"
if test_file "$SCRIPT"; then
    [ -x "$SCRIPT" ] && pass "prepare-binaries.sh: executable" || fail "Executable" "not executable"
    grep -q 'DEBUG' "$SCRIPT" && pass "prepare-binaries.sh: debug support" || fail "Debug" "missing"
    grep -q 'command -v cargo' "$SCRIPT" && pass "prepare-binaries.sh: cargo check" || fail "Cargo check" "missing"
    grep -q 'rustup target list' "$SCRIPT" && pass "prepare-binaries.sh: target verify" || fail "Target verify" "missing"
    grep -q 'Error:' "$SCRIPT" && pass "prepare-binaries.sh: error messages" || fail "Errors" "missing"
fi

# Test Linux scripts
echo ""
echo "Testing Linux installer scripts..."
for script in "$TEST_DIR/../linux"/*.sh; do
    test_file "$script" || true  # Continue on failure
done

# Test macOS scripts
echo ""
echo "Testing macOS installer scripts..."
for script in "$TEST_DIR/../macos"/*.sh; do
    test_file "$script" || true  # Continue on failure
done

# Test Windows NSIS
echo ""
echo "Testing Windows NSIS hooks..."
NSIS="$TEST_DIR/../windows/hooks.nsh"
if [ -f "$NSIS" ]; then
    pass "hooks.nsh: exists"
    grep -q 'StrFunc.nsh' "$NSIS" && pass "hooks.nsh: includes StrFunc" || fail "StrFunc" "missing"
    grep -q 'NSIS_HOOK_POSTINSTALL' "$NSIS" && pass "hooks.nsh: postinstall" || fail "Postinstall" "missing"
    grep -q 'NSIS_HOOK_PREUNINSTALL' "$NSIS" && pass "hooks.nsh: preuninstall" || fail "Preuninstall" "missing"
else
    fail "hooks.nsh" "not found"
fi

# Summary
echo ""
echo "========================================"
echo "Results: ${GREEN}$TESTS_PASSED passed${NC}, ${RED}$TESTS_FAILED failed${NC}"
echo "========================================"

[ $TESTS_FAILED -eq 0 ] && exit 0 || exit 1
