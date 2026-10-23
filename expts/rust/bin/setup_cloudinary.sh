#!/usr/bin/env bash
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Load .env if present
if [ -f "${ROOT_DIR}/.env" ]; then
    echo "==> Loading environment from ${ROOT_DIR}/.env"
    set -a
    # shellcheck disable=SC1091
    source "${ROOT_DIR}/.env"
    set +a
fi

# Verify required credentials
if [ -z "${CLOUDINARY_CLOUD_NAME:-}" ] || [ -z "${CLOUDINARY_API_KEY:-}" ] || [ -z "${CLOUDINARY_API_SECRET:-}" ]; then
    echo "ERROR: Missing required Cloudinary credentials."
    echo "Ensure CLOUDINARY_CLOUD_NAME, CLOUDINARY_API_KEY, and CLOUDINARY_API_SECRET are set."
    exit 1
fi

echo "========================================================"
echo " Cloudinary Named Transformations Configuration"
echo " Account Cloud Name: ${CLOUDINARY_CLOUD_NAME}"
echo "========================================================"

# Ordered list of transformations according to haven-ui specification
TRANSFORMS=(
    "avatar:c_fill,g_face,w_96,h_96,f_webp:Profile avatar (1:1 standard display)"
    "avatar_2x:c_fill,g_face,w_192,h_192,f_webp:Profile avatar (1:1 Retina display)"
    "thumb:c_fill,w_120,h_90,f_webp:Offer management row thumbnail (4:3 standard display)"
    "thumb_2x:c_fill,w_240,h_180,f_webp:Offer management row thumbnail (4:3 Retina display)"
    "card:c_fill,w_320,h_240,f_webp:Offer card cover (4:3 standard display)"
    "card_2x:c_fill,w_640,h_480,f_webp:Offer card cover (4:3 Retina display)"
    "detail:c_fill,w_640,h_480,f_webp:Offer detail view photo stack (4:3 standard display)"
    "detail_2x:c_fill,w_960,h_720,f_webp:Offer detail view photo stack (4:3 Retina display)"
)

AUTH="${CLOUDINARY_API_KEY}:${CLOUDINARY_API_SECRET}"
BASE_URL="https://api.cloudinary.com/v1_1/${CLOUDINARY_CLOUD_NAME}/transformations"

total=${#TRANSFORMS[@]}
current=0

for item in "${TRANSFORMS[@]}"; do
    current=$((current + 1))
    IFS=":" read -r name transform description <<< "${item}"

    echo ""
    echo "--------------------------------------------------------"
    echo "[$current/$total] Configuring Named Transformation: '$name'"
    echo "  Description: $description"
    echo "  Definition:  $transform"
    echo "--------------------------------------------------------"

    # 1. Create or update the transformation definition
    echo "==> [1/2] Creating/updating transformation '$name' via Cloudinary Admin API..."
    create_response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST \
        -u "${AUTH}" \
        -d "transformation=${transform}" \
        "${BASE_URL}/${name}")

    create_status=$(echo "${create_response}" | grep "HTTP_STATUS:" | cut -d':' -f2)
    create_body=$(echo "${create_response}" | grep -v "HTTP_STATUS:")

    if [ "${create_status}" -ge 200 ] && [ "${create_status}" -lt 300 ]; then
        echo "    SUCCESS (HTTP ${create_status}): Transformation '$name' created."
        echo "    Response: ${create_body}"
    else
        echo "    Transformation '$name' returned HTTP ${create_status}. Attempting update via PUT..."
        update_response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X PUT \
            -u "${AUTH}" \
            -d "transformation=${transform}" \
            "${BASE_URL}/${name}")
        update_status=$(echo "${update_response}" | grep "HTTP_STATUS:" | cut -d':' -f2)
        update_body=$(echo "${update_response}" | grep -v "HTTP_STATUS:")

        if [ "${update_status}" -ge 200 ] && [ "${update_status}" -lt 300 ]; then
            echo "    SUCCESS (HTTP ${update_status}): Transformation '$name' updated."
            echo "    Response: ${update_body}"
        else
            echo "    ERROR (HTTP ${update_status}): Failed to create or update transformation '$name'."
            echo "    Response: ${update_body}"
            exit 1
        fi
    fi

    # 2. Flag as allowed for strict transformations
    echo "==> [2/2] Whitelisting '$name' for Strict Transformations (allowed_for_strict=true)..."
    strict_response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X PUT \
        -u "${AUTH}" \
        -d "allowed_for_strict=true" \
        "${BASE_URL}/${name}")

    strict_status=$(echo "${strict_response}" | grep "HTTP_STATUS:" | cut -d':' -f2)
    strict_body=$(echo "${strict_response}" | grep -v "HTTP_STATUS:")

    if [ "${strict_status}" -ge 200 ] && [ "${strict_status}" -lt 300 ]; then
        echo "    SUCCESS (HTTP ${strict_status}): Transformation '$name' allowed for strict mode."
        echo "    Response: ${strict_body}"
    else
        echo "    ERROR (HTTP ${strict_status}): Failed to update strict flag for '$name'."
        echo "    Response: ${strict_body}"
        exit 1
    fi
done

echo ""
echo "========================================================"
echo " Account Security Requirement Notice"
echo "========================================================"
echo "Note: Named transformations are whitelisted for strict mode."
echo "Ensure 'Strict Transformations' is toggled ON under:"
echo "Cloudinary Console -> Settings -> Security -> Strict Transformations"
echo "========================================================"

echo ""
echo "========================================================"
echo " Verification: Testing live delivery of named transformation"
echo "========================================================"

# Allow passing an asset public_id to test, or fallback to Cloudinary sample
TEST_ASSET_ID="${1:-sample}"
TEST_URL="https://res.cloudinary.com/${CLOUDINARY_CLOUD_NAME}/image/upload/t_detail/${TEST_ASSET_ID}.jpg"
echo "Testing GET request against:"
echo "  ${TEST_URL}"

test_headers=$(curl -s -I "${TEST_URL}")
echo "${test_headers}"

if echo "${test_headers}" | grep -qE "HTTP/[12](\.[0-9])? 200"; then
    echo ""
    echo "==> ALL CHECKS PASSED! HTTP 200 received from Cloudinary."
    echo "    Named transformation 'detail' is active and working."
else
    echo ""
    if [ -n "${1:-}" ]; then
        echo "==> ERROR: Verification failed for supplied asset '${TEST_ASSET_ID}'."
        exit 1
    else
        echo "==> Note: Test URL did not return HTTP 200. If '${TEST_ASSET_ID}' does not exist"
        echo "    in your account, test with an existing asset: ./setup_cloudinary.sh <public_id>"
    fi
fi
