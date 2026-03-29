#!/bin/bash
set -e

# Usage: ./scripts/export_backers.sh <CONTRACT_ID> [output_file]
CONTRACT_ID=${1:?Usage: export_backers.sh <CONTRACT_ID> [output_file]}
OUTPUT_FILE=${2:-backers_$(date +%Y%m%d_%H%M%S).csv}
NETWORK="testnet"

echo "Fetching contributors from contract: $CONTRACT_ID"
echo "Network: $NETWORK"
echo ""

# Fetch the list of contributor addresses
CONTRIBUTORS=$(soroban contract invoke \
  --id "$CONTRACT_ID" \
  --network "$NETWORK" \
  --source "$(soroban keys address default)" \
  -- contributors)

# Write CSV header
echo "address,amount" > "$OUTPUT_FILE"

# Parse the JSON array of addresses and fetch each balance
echo "$CONTRIBUTORS" | jq -r '.[]' | while read -r ADDRESS; do
  AMOUNT=$(soroban contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --source "$(soroban keys address default)" \
    -- contribution \
    --contributor "$ADDRESS")
  
  echo "$ADDRESS,$AMOUNT" >> "$OUTPUT_FILE"
  echo "  Exported: $ADDRESS → $AMOUNT"
done

echo ""
echo "Export complete: $OUTPUT_FILE"
echo "Total contributors: $(tail -n +2 "$OUTPUT_FILE" | wc -l)"
