#!/bin/bash
#
# Trigger prerelease.yml workflow and monitor progress
# Usage: ./trigger-prerelease.sh [version_tag] [platform]
#

set -euo pipefail

# Configuration
ORG="BTCDecoded"
REPO="bllvm"
WORKFLOW_FILE="prerelease.yml"
TOKEN="${GITHUB_TOKEN:-}"
VERSION_TAG="${1:-v0.2.0-prerelease}"
PLATFORM="${2:-linux}"

# Validate token
if [ -z "$TOKEN" ]; then
    echo "Error: GITHUB_TOKEN environment variable is required"
    echo "Usage: GITHUB_TOKEN=your_token ./trigger-prerelease.sh [version_tag] [platform]"
    exit 1
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}🚀 Triggering Prerelease Workflow${NC}"
echo "====================================="
echo "Repository: ${ORG}/${REPO}"
echo "Workflow: ${WORKFLOW_FILE}"
echo "Version: ${VERSION_TAG}"
echo "Platform: ${PLATFORM}"
echo ""

# Step 1: Get workflow ID
echo -e "${CYAN}📋 Getting workflow ID...${NC}"
WORKFLOWS_JSON=$(curl -s -H "Authorization: token ${TOKEN}" \
    -H "Accept: application/vnd.github.v3+json" \
    "https://api.github.com/repos/${ORG}/${REPO}/actions/workflows")

WORKFLOW_ID=$(echo "$WORKFLOWS_JSON" | jq -r ".workflows[] | select(.path == \".github/workflows/${WORKFLOW_FILE}\") | .id")

if [ -z "$WORKFLOW_ID" ] || [ "$WORKFLOW_ID" = "null" ]; then
    echo -e "${RED}❌ Workflow ${WORKFLOW_FILE} not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Found workflow ID: ${WORKFLOW_ID}${NC}"
echo ""

# Step 2: Trigger workflow
echo -e "${CYAN}🎯 Triggering workflow...${NC}"
TRIGGER_RESPONSE=$(curl -s -X POST \
    -H "Authorization: token ${TOKEN}" \
    -H "Accept: application/vnd.github.v3+json" \
    "https://api.github.com/repos/${ORG}/${REPO}/actions/workflows/${WORKFLOW_ID}/dispatches" \
    -d "{
        \"ref\": \"main\",
        \"inputs\": {
            \"version_tag\": \"${VERSION_TAG}\",
            \"platform\": \"${PLATFORM}\"
        }
    }")

# Check if trigger was successful (204 No Content means success)
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
    -H "Authorization: token ${TOKEN}" \
    -H "Accept: application/vnd.github.v3+json" \
    "https://api.github.com/repos/${ORG}/${REPO}/actions/workflows/${WORKFLOW_ID}/dispatches" \
    -d "{
        \"ref\": \"main\",
        \"inputs\": {
            \"version_tag\": \"${VERSION_TAG}\",
            \"platform\": \"${PLATFORM}\"
        }
    }")

if [ "$HTTP_CODE" = "204" ]; then
    echo -e "${GREEN}✅ Workflow triggered successfully!${NC}"
else
    echo -e "${RED}❌ Failed to trigger workflow (HTTP ${HTTP_CODE})${NC}"
    echo "Response: ${TRIGGER_RESPONSE}"
    exit 1
fi

echo ""
echo -e "${YELLOW}⏳ Waiting 10 seconds for workflow to start...${NC}"
sleep 10

# Step 3: Get the triggered run
echo -e "${CYAN}🔍 Finding workflow run...${NC}"
MAX_ATTEMPTS=5
ATTEMPT=0
RUN_ID=""

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    RUNS_JSON=$(curl -s -H "Authorization: token ${TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${ORG}/${REPO}/actions/workflows/${WORKFLOW_ID}/runs?per_page=5")
    
    # Find the most recent run with our version tag
    RUN_ID=$(echo "$RUNS_JSON" | jq -r ".workflow_runs[] | select(.display_title | contains(\"${VERSION_TAG}\")) | .id" | head -1)
    
    if [ -n "$RUN_ID" ] && [ "$RUN_ID" != "null" ]; then
        break
    fi
    
    ATTEMPT=$((ATTEMPT + 1))
    if [ $ATTEMPT -lt $MAX_ATTEMPTS ]; then
        echo "  Attempt ${ATTEMPT}/${MAX_ATTEMPTS} - waiting 5 seconds..."
        sleep 5
    fi
done

if [ -z "$RUN_ID" ] || [ "$RUN_ID" = "null" ]; then
    echo -e "${YELLOW}⚠️  Could not find workflow run, but workflow was triggered${NC}"
    echo "Check manually: https://github.com/${ORG}/${REPO}/actions"
    exit 0
fi

echo -e "${GREEN}✅ Found workflow run ID: ${RUN_ID}${NC}"
echo ""

# Step 4: Monitor workflow
echo -e "${BLUE}📊 Monitoring workflow progress...${NC}"
echo "====================================="
echo "Run URL: https://github.com/${ORG}/${REPO}/actions/runs/${RUN_ID}"
echo ""
echo "Press Ctrl+C to stop monitoring (workflow will continue)"
echo ""

POLL_INTERVAL=30  # Check every 30 seconds
LAST_STATUS=""

while true; do
    RUN_INFO=$(curl -s -H "Authorization: token ${TOKEN}" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${ORG}/${REPO}/actions/runs/${RUN_ID}")
    
    STATUS=$(echo "$RUN_INFO" | jq -r '.status')
    CONCLUSION=$(echo "$RUN_INFO" | jq -r '.conclusion // "pending"')
    HTML_URL=$(echo "$RUN_INFO" | jq -r '.html_url')
    
    if [ "$STATUS" != "$LAST_STATUS" ]; then
        TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
        echo -e "[${TIMESTAMP}] ${CYAN}Status: ${STATUS}${NC} ${CONCLUSION:+(${CONCLUSION})}"
        LAST_STATUS="$STATUS"
    fi
    
    if [ "$STATUS" = "completed" ]; then
        echo ""
        echo "====================================="
        if [ "$CONCLUSION" = "success" ]; then
            echo -e "${GREEN}✅ Workflow completed successfully!${NC}"
            echo ""
            echo "Checking for release artifacts..."
            
            # Check for release
            RELEASE_INFO=$(curl -s -H "Authorization: token ${TOKEN}" \
                -H "Accept: application/vnd.github.v3+json" \
                "https://api.github.com/repos/${ORG}/${REPO}/releases/tags/${VERSION_TAG}" || echo "{}")
            
            if echo "$RELEASE_INFO" | jq -e '.id' > /dev/null 2>&1; then
                ASSETS=$(echo "$RELEASE_INFO" | jq -r '.assets[]?.name' | grep -E "(bllvm|experimental)" || true)
                if [ -n "$ASSETS" ]; then
                    echo -e "${GREEN}✅ Release found with artifacts:${NC}"
                    echo "$ASSETS" | while read asset; do
                        echo "  - $asset"
                    done
                else
                    echo -e "${YELLOW}⚠️  Release found but no artifacts detected${NC}"
                fi
                RELEASE_URL=$(echo "$RELEASE_INFO" | jq -r '.html_url')
                echo ""
                echo "Release URL: ${RELEASE_URL}"
            else
                echo -e "${YELLOW}⚠️  Release not found yet (may take a few moments)${NC}"
            fi
            
        else
            echo -e "${RED}❌ Workflow failed with conclusion: ${CONCLUSION}${NC}"
            echo ""
            echo "Download logs:"
            echo "curl -L -H \"Authorization: token ${TOKEN}\" \\"
            echo "  https://api.github.com/repos/${ORG}/${REPO}/actions/runs/${RUN_ID}/logs -o logs.zip"
        fi
        echo ""
        echo "Workflow URL: ${HTML_URL}"
        break
    fi
    
    sleep $POLL_INTERVAL
done

