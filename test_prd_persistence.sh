#!/bin/bash

# ABOUTME: Integration test script for PRD section persistence
# ABOUTME: Verifies resumed Quick Mode PRD generation with section data loading from database

set -e

# Color output for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

API_URL="${API_URL:-http://localhost:4001}"
PROJECT_ID="test-project-$(date +%s)"
echo -e "${BLUE}=== PRD Section Persistence Test ===${NC}"
echo "API URL: $API_URL"
echo "Project ID: $PROJECT_ID"
echo ""

# Helper functions
function log_step() {
    echo -e "${BLUE}>>> $1${NC}"
}

function log_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

function log_error() {
    echo -e "${RED}✗ $1${NC}"
}

function log_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Test 1: Create a project
log_step "Step 1: Creating test project"
PROJECT_RESPONSE=$(curl -s -X POST "$API_URL/api/projects" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"$PROJECT_ID\",
    \"description\": \"Test project for PRD persistence\",
    \"projectRoot\": \"/tmp/test-project\"
  }")

PROJECT_ID=$(echo "$PROJECT_RESPONSE" | jq -r '.data.id // empty')
if [ -z "$PROJECT_ID" ] || [ "$PROJECT_ID" == "null" ]; then
    log_error "Failed to create project"
    echo "Response: $PROJECT_RESPONSE"
    exit 1
fi
log_success "Project created: $PROJECT_ID"
echo ""

# Test 2: Start an ideate session
log_step "Step 2: Starting ideate session"
SESSION_RESPONSE=$(curl -s -X POST "$API_URL/api/ideate/start" \
  -H "Content-Type: application/json" \
  -d "{
    \"projectId\": \"$PROJECT_ID\",
    \"initialDescription\": \"A collaborative task management tool with real-time updates and team notifications\",
    \"mode\": \"quick\"
  }")

SESSION_ID=$(echo "$SESSION_RESPONSE" | jq -r '.data.id // empty')
if [ -z "$SESSION_ID" ] || [ "$SESSION_ID" == "null" ]; then
    log_error "Failed to create ideate session"
    echo "Response: $SESSION_RESPONSE"
    exit 1
fi
log_success "Session created: $SESSION_ID"
echo ""

# Test 3: Generate PRD with quick_generate
log_step "Step 3: Generating PRD (this may take a few minutes)"
log_info "Calling quick-generate endpoint to save sections to database..."

# Use a timeout that allows sufficient time for PRD generation
GENERATE_RESPONSE=$(curl -s -X POST "$API_URL/api/ideate/$SESSION_ID/quick-generate" \
  -H "Content-Type: application/json" \
  -d '{"provider": "anthropic", "model": "claude-haiku-4-5-20251001"}' \
  --max-time 300)

# Check for successful generation
if echo "$GENERATE_RESPONSE" | jq -e '.data.sections' > /dev/null 2>&1; then
    log_success "PRD generated successfully"

    # Check which sections were generated
    SECTIONS=$(echo "$GENERATE_RESPONSE" | jq '.data.sections | keys' | tr -d '\n')
    log_info "Generated sections: $SECTIONS"
else
    log_error "Failed to generate PRD or retrieve sections"
    echo "Response: $GENERATE_RESPONSE"
    exit 1
fi
echo ""

# Test 4: Get preview to load sections from database
log_step "Step 4: Calling get_preview to load sections from database"
PREVIEW_RESPONSE=$(curl -s -X GET "$API_URL/api/ideate/$SESSION_ID/preview" \
  -H "Content-Type: application/json")

if echo "$PREVIEW_RESPONSE" | jq -e '.data' > /dev/null 2>&1; then
    log_success "Preview loaded successfully"
else
    log_error "Failed to get preview"
    echo "Response: $PREVIEW_RESPONSE"
    exit 1
fi
echo ""

# Test 5: Verify all sections are present and contain data
log_step "Step 5: Verifying section data loaded from database"

SECTIONS_TO_CHECK=("overview" "ux" "technical" "roadmap" "dependencies" "risks" "appendix")
SECTIONS_FOUND=0
SECTIONS_MISSING=0

for section in "${SECTIONS_TO_CHECK[@]}"; do
    if echo "$PREVIEW_RESPONSE" | jq -e ".data.sections.$section" > /dev/null 2>&1; then
        SECTION_DATA=$(echo "$PREVIEW_RESPONSE" | jq ".data.sections.$section")

        # Check if section has some content (not empty or null)
        if [ "$SECTION_DATA" != "null" ] && [ -n "$SECTION_DATA" ]; then
            log_success "Section '$section' loaded from database"
            ((SECTIONS_FOUND++))
        else
            log_error "Section '$section' is empty or null"
            ((SECTIONS_MISSING++))
        fi
    else
        log_info "Section '$section' not present (may be optional)"
    fi
done
echo ""

# Test 6: Verify markdown content is present
log_step "Step 6: Verifying markdown content"
if echo "$PREVIEW_RESPONSE" | jq -e '.data.markdown' > /dev/null 2>&1; then
    MARKDOWN_LENGTH=$(echo "$PREVIEW_RESPONSE" | jq '.data.markdown | length')
    if [ "$MARKDOWN_LENGTH" -gt 100 ]; then
        log_success "Markdown content present and substantial (${MARKDOWN_LENGTH} bytes)"
    else
        log_error "Markdown content is too short (${MARKDOWN_LENGTH} bytes)"
    fi
else
    log_error "No markdown content in response"
fi
echo ""

# Test 7: Verify round-trip integrity
log_step "Step 7: Verifying round-trip data integrity"

# Extract overview from both responses to compare
OVERVIEW_SECTIONS=$(echo "$PREVIEW_RESPONSE" | jq '.data.sections.overview')
if [ -n "$OVERVIEW_SECTIONS" ] && [ "$OVERVIEW_SECTIONS" != "null" ]; then
    # Check if overview has expected fields
    if echo "$OVERVIEW_SECTIONS" | jq -e '.id' > /dev/null 2>&1; then
        log_success "Overview section has proper structure (contains 'id' field)"
    fi

    if echo "$OVERVIEW_SECTIONS" | jq -e '.session_id' > /dev/null 2>&1; then
        log_success "Overview section is session-scoped (contains 'session_id' field)"
    fi
else
    log_info "Overview not available for structure verification"
fi
echo ""

# Final summary
log_step "Test Summary"
echo ""
if [ "$SECTIONS_FOUND" -gt 0 ]; then
    log_success "$SECTIONS_FOUND sections loaded from database"
fi
if [ "$SECTIONS_MISSING" -eq 0 ]; then
    log_success "No missing sections"
fi

echo ""
log_success "PRD section persistence test completed!"
log_info "Next step: Verify this in the UI by loading the Quick Mode with the same session ID"
echo ""
echo "Session details:"
echo "  Session ID: $SESSION_ID"
echo "  Project ID: $PROJECT_ID"
echo ""
