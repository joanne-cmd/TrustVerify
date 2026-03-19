#!/bin/bash
# Integration test for PPID Verification Dashboard
# Run with backend already running on default port (e.g., 8080)
#
# Update TRUSTED_QUOTE and UNKNOWN_QUOTE once you have real mock data:
#   export TRUSTED_QUOTE="$(cat samples/trusted.hex)"
#   export UNKNOWN_QUOTE="$(cat samples/unknown.hex)"
#   ./scripts/integration_test.sh

set -e

# Run from project root: ./scripts/integration_test.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

API_URL="${API_URL:-http://localhost:8080}"
TRUSTED_QUOTE="${TRUSTED_QUOTE:-$(cat samples/trusted.hex 2>/dev/null || echo '0x0000000000000001')}"
UNKNOWN_QUOTE="${UNKNOWN_QUOTE:-$(cat samples/unknown.hex 2>/dev/null || echo '0xffffffffffffffff')}"
INVALID_QUOTE="not-valid-hex"

echo "=== TrustVerify — Integration Tests ==="
echo "API URL: $API_URL"
echo ""

# Test 1: Valid/Trusted quote
echo "Test 1: Trusted quote (in registry)"
RESP=$(curl -s -X POST "$API_URL/api/verify" \
  -H "Content-Type: application/json" \
  -d "{\"quote\": \"$TRUSTED_QUOTE\", \"format\": \"intel_dcap\"}")
if echo "$RESP" | grep -q '"valid":true\|"provider_match":{.*"found":true'; then
  echo "  PASS: Trusted quote returns valid/provider match"
else
  echo "  FAIL: Expected valid/provider match. Got: $RESP"
  exit 1
fi

# Test 2: Unknown quote (not in registry)
echo "Test 2: Unknown quote (not in registry)"
RESP=$(curl -s -X POST "$API_URL/api/verify" \
  -H "Content-Type: application/json" \
  -d "{\"quote\": \"$UNKNOWN_QUOTE\", \"format\": \"intel_dcap\"}")
if echo "$RESP" | grep -q '"provider_match":{.*"found":false\|"valid":true'; then
  echo "  PASS: Unknown quote returns found:false or valid"
else
  echo "  FAIL: Expected unknown. Got: $RESP"
  exit 1
fi

# Test 3: Invalid quote
echo "Test 3: Invalid quote"
RESP=$(curl -s -X POST "$API_URL/api/verify" \
  -H "Content-Type: application/json" \
  -d "{\"quote\": \"$INVALID_QUOTE\", \"format\": \"intel_dcap\"}")
if echo "$RESP" | grep -q '"valid":false\|"error"\|400'; then
  echo "  PASS: Invalid quote rejected"
else
  echo "  FAIL: Expected invalid/error. Got: $RESP"
  exit 1
fi

echo ""
echo "=== All integration tests passed ==="
