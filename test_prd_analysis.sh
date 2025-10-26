#!/bin/bash

# Test PRD Analysis with Database Persistence

echo "🧪 Testing PRD Analysis..."
echo ""

# Get PRD details
PRD_ID="pGxbIbUg"
PROJECT_ID="PaxXb32Q"

# Create the request payload
cat > /tmp/prd_request.json <<EOF
{
  "prdId": "$PRD_ID",
  "contentMarkdown": "A simple javascript tic-tac-toe game.",
  "provider": "anthropic",
  "model": "claude-3-5-haiku-20241022"
}
EOF

echo "📝 Analyzing PRD: $PRD_ID"
echo "📦 Project: $PROJECT_ID"
echo ""

# Call the API
echo "🚀 Calling analyze-prd endpoint..."
RESPONSE=$(curl -s -X POST http://localhost:4001/api/ai/analyze-prd \
  -H "Content-Type: application/json" \
  -d @/tmp/prd_request.json)

# Check if response contains success
if echo "$RESPONSE" | grep -q '"success":true'; then
  echo "✅ API call succeeded!"
  echo ""
  
  # Wait a moment for database writes
  sleep 2
  
  # Check database
  echo "🔍 Checking database for saved data..."
  echo ""
  
  CAPS=$(sqlite3 ~/.orkee/orkee.db "SELECT COUNT(*) FROM spec_capabilities WHERE prd_id='$PRD_ID' AND deleted_at IS NULL;")
  REQS=$(sqlite3 ~/.orkee/orkee.db "SELECT COUNT(*) FROM spec_requirements;")
  SCENS=$(sqlite3 ~/.orkee/orkee.db "SELECT COUNT(*) FROM spec_scenarios;")
  TASKS=$(sqlite3 ~/.orkee/orkee.db "SELECT COUNT(*) FROM tasks WHERE category IS NOT NULL;")
  
  echo "📊 Results:"
  echo "   Capabilities: $CAPS"
  echo "   Requirements: $REQS"
  echo "   Scenarios: $SCENS"
  echo "   Tasks: $TASKS"
  echo ""
  
  if [ "$CAPS" -gt 0 ]; then
    echo "🎉 SUCCESS! Data was saved to database!"
    echo ""
    echo "📋 Capability details:"
    sqlite3 ~/.orkee/orkee.db "SELECT name, requirement_count FROM spec_capabilities WHERE prd_id='$PRD_ID' AND deleted_at IS NULL;"
  else
    echo "❌ FAILED! No capabilities were saved."
    echo ""
    echo "Response preview:"
    echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
  fi
else
  echo "❌ API call failed!"
  echo ""
  echo "Response:"
  echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
fi

# Cleanup
rm -f /tmp/prd_request.json
