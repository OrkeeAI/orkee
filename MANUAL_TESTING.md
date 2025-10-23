# Manual Testing Procedures - API Token Authentication

This document provides step-by-step manual testing procedures for verifying the API token authentication system.

## Prerequisites

- Fresh Orkee installation or ability to delete `~/.orkee/` directory
- `curl` command available
- Orkee CLI installed

## Test 1: First Startup - Token Generation

**Objective**: Verify that API token is generated and logged on first startup.

**Steps**:
1. Clean up any existing Orkee data:
   ```bash
   rm -rf ~/.orkee/
   ```

2. Start Orkee dashboard:
   ```bash
   orkee dashboard
   ```

3. **Expected Results**:
   - Server starts successfully
   - Console output includes a message like:
     ```
     ============================================================
     API TOKEN (save this - shown once):
     <long-token-string>
     ============================================================
     WARNING: This token is required for API access.
     Token saved to: /Users/<username>/.orkee/api-token
     ```
   - Token file is created at `~/.orkee/api-token`

4. Verify token file:
   ```bash
   cat ~/.orkee/api-token
   ```
   - Should display the same token shown in console

**✅ PASS Criteria**:
- Token is generated and displayed once
- Token file exists with correct permissions (0600)
- Server starts successfully

---

## Test 2: API Call Without Token

**Objective**: Verify that API calls without token return 401 Unauthorized.

**Steps**:
1. Start Orkee dashboard (if not already running)

2. Try to access a protected endpoint without token:
   ```bash
   curl -v http://localhost:4001/api/projects
   ```

3. **Expected Results**:
   - HTTP status: `401 Unauthorized`
   - Response body includes:
     ```json
     {
       "success": false,
       "error": "API token required. Please include X-API-Token header.",
       "request_id": "<some-id>"
     }
     ```

**✅ PASS Criteria**:
- Returns 401 status code
- Clear error message about missing token
- Request ID is included for debugging

---

## Test 3: API Call With Valid Token

**Objective**: Verify that API calls with valid token succeed.

**Steps**:
1. Read the token from file:
   ```bash
   export TOKEN=$(cat ~/.orkee/api-token)
   ```

2. Make authenticated API call:
   ```bash
   curl -H "X-API-Token: $TOKEN" http://localhost:4001/api/projects
   ```

3. **Expected Results**:
   - HTTP status: `200 OK`
   - Response body includes project list:
     ```json
     {
       "success": true,
       "data": [],
       "error": null
     }
     ```

**✅ PASS Criteria**:
- Returns 200 status code
- Response includes project data
- No authentication errors

---

## Test 4: API Call With Invalid Token

**Objective**: Verify that API calls with invalid token return 401 Unauthorized.

**Steps**:
1. Try to access endpoint with wrong token:
   ```bash
   curl -v -H "X-API-Token: invalid-token-12345" http://localhost:4001/api/projects
   ```

2. **Expected Results**:
   - HTTP status: `401 Unauthorized`
   - Response body:
     ```json
     {
       "success": false,
       "error": "Invalid API token",
       "request_id": "<some-id>"
     }
     ```

**✅ PASS Criteria**:
- Returns 401 status code
- Clear error message about invalid token

---

## Test 5: Whitelisted Endpoints Work Without Token

**Objective**: Verify that health and status endpoints don't require authentication.

**Steps**:
1. Test health endpoint without token:
   ```bash
   curl http://localhost:4001/api/health
   ```

2. Test status endpoint without token:
   ```bash
   curl http://localhost:4001/api/status
   ```

3. **Expected Results**:
   - Both endpoints return `200 OK`
   - No authentication errors
   - Health endpoint returns: `{"status":"healthy"}`

**✅ PASS Criteria**:
- Both endpoints accessible without token
- Return expected responses

---

## Test 6: Cannot Modify Environment-Only Settings

**Objective**: Verify that is_env_only settings cannot be modified via API.

**Steps**:
1. Get valid token:
   ```bash
   export TOKEN=$(cat ~/.orkee/api-token)
   ```

2. Try to modify `api_port` (environment-only setting):
   ```bash
   curl -X PUT \
     -H "X-API-Token: $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"value": "8080"}' \
     http://localhost:4001/api/settings/api_port
   ```

3. **Expected Results**:
   - HTTP status: `403 Forbidden`
   - Response body:
     ```json
     {
       "success": false,
       "error": "Cannot modify environment-only setting: api_port",
       "request_id": "<some-id>"
     }
     ```

4. Repeat for `ui_port` and `dev_mode`

**✅ PASS Criteria**:
- Returns 403 Forbidden
- Clear error message about environment-only restriction
- Settings remain unchanged

---

## Test 7: Invalid Port Number Validation

**Objective**: Verify that invalid port values are rejected with proper validation errors.

**Steps**:
1. Try to set invalid port (too low):
   ```bash
   export TOKEN=$(cat ~/.orkee/api-token)
   curl -X PUT \
     -H "X-API-Token: $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"value": "0"}' \
     http://localhost:4001/api/settings/ui_port
   ```

2. Try to set invalid port (too high):
   ```bash
   curl -X PUT \
     -H "X-API-Token: $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"value": "65536"}' \
     http://localhost:4001/api/settings/ui_port
   ```

3. Try to set non-numeric port:
   ```bash
   curl -X PUT \
     -H "X-API-Token: $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"value": "abc"}' \
     http://localhost:4001/api/settings/ui_port
   ```

4. **Expected Results** (for all attempts):
   - HTTP status: `400 Bad Request` or `403 Forbidden`
   - Response includes validation error message

**✅ PASS Criteria**:
- All invalid values rejected
- Clear validation error messages
- Settings remain unchanged

---

## Test 8: Bulk Update With Invalid Value

**Objective**: Verify that bulk updates with one invalid value rollback all changes.

**Steps**:
1. Get valid token:
   ```bash
   export TOKEN=$(cat ~/.orkee/api-token)
   ```

2. Get current value of `cloud_enabled`:
   ```bash
   curl -H "X-API-Token: $TOKEN" http://localhost:4001/api/settings/cloud_enabled
   # Note the current value (probably "false")
   ```

3. Try bulk update with one invalid value:
   ```bash
   curl -X POST \
     -H "X-API-Token: $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{
       "settings": [
         {"key": "cloud_enabled", "value": "true"},
         {"key": "rate_limit_health_rpm", "value": "0"}
       ]
     }' \
     http://localhost:4001/api/settings/bulk-update
   ```

4. **Expected Results**:
   - HTTP status: `400 Bad Request`
   - Response includes validation error for `rate_limit_health_rpm`

5. Verify rollback - check `cloud_enabled` again:
   ```bash
   curl -H "X-API-Token: $TOKEN" http://localhost:4001/api/settings/cloud_enabled
   ```

6. **Expected Results**:
   - `cloud_enabled` still has original value (not updated to "true")
   - Transaction was rolled back

**✅ PASS Criteria**:
- Bulk update fails
- No settings are updated (all-or-nothing)
- Original values remain unchanged

---

## Test 9: Dashboard Authentication

**Objective**: Verify that the dashboard can authenticate with the token.

**Steps**:
1. Start Orkee dashboard (if not already running):
   ```bash
   orkee dashboard
   ```

2. Open browser to: `http://localhost:5173`

3. Navigate to Projects tab

4. **Expected Results**:
   - Projects page loads successfully
   - No authentication errors in browser console
   - Can perform project operations (create, edit, delete)

5. Check browser network tab:
   - All API requests should include `X-API-Token` header
   - All requests return 200 OK (not 401)

**✅ PASS Criteria**:
- Dashboard loads and functions normally
- No authentication errors
- API requests include token header

---

## Test 10: Settings Persistence Across Restarts

**Objective**: Verify that settings changes persist after server restart.

**Steps**:
1. Get valid token:
   ```bash
   export TOKEN=$(cat ~/.orkee/api-token)
   ```

2. Update a non-env-only setting:
   ```bash
   curl -X PUT \
     -H "X-API-Token: $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"value": "true"}' \
     http://localhost:4001/api/settings/telemetry_enabled
   ```

3. Verify the change:
   ```bash
   curl -H "X-API-Token: $TOKEN" http://localhost:4001/api/settings/telemetry_enabled
   # Should return "true"
   ```

4. Stop the Orkee server (Ctrl+C)

5. Restart the server:
   ```bash
   orkee dashboard
   ```

6. Read token (same token should exist):
   ```bash
   export TOKEN=$(cat ~/.orkee/api-token)
   ```

7. Check the setting again:
   ```bash
   curl -H "X-API-Token: $TOKEN" http://localhost:4001/api/settings/telemetry_enabled
   ```

8. **Expected Results**:
   - Setting still has value "true"
   - Same token works after restart
   - No data loss

**✅ PASS Criteria**:
- Settings persist across restarts
- Token remains valid
- No database corruption

---

## Test 11: Token Rotation (Future Feature)

**Note**: This test is for future implementation when token rotation UI is added.

**Objective**: Verify that regenerating a token invalidates the old token.

**Steps** (when implemented):
1. Get current token
2. Use token to make successful API call
3. Regenerate token via dashboard UI
4. Try to use old token - should fail with 401
5. Use new token - should succeed

---

## Troubleshooting

### Token File Missing
If `~/.orkee/api-token` is missing:
1. Stop Orkee server
2. Delete `~/.orkee/orkee.db` to reset database
3. Restart Orkee server
4. New token will be generated

### Permission Denied on Token File
Fix file permissions:
```bash
chmod 600 ~/.orkee/api-token
```

### Database Locked Errors
Stop all Orkee processes:
```bash
pkill orkee
```

Then restart:
```bash
orkee dashboard
```

---

## Test Summary Checklist

After completing all tests, verify:

- [x] Token generated on first startup
- [x] Token file created with correct permissions
- [x] API calls without token return 401
- [x] API calls with valid token succeed
- [x] API calls with invalid token return 401
- [x] Whitelisted endpoints work without token
- [x] Cannot modify environment-only settings (403)
- [x] Invalid values rejected with proper errors (400)
- [x] Bulk updates are atomic (all-or-nothing)
- [x] Dashboard authenticates successfully
- [x] Settings persist across restarts

**All tests passing = Authentication system is working correctly ✅**
