#!/bin/bash
set -e

# Usage: ./scripts/deploy.sh <creator> <token> <goal> <deadline> <min_contribution>
# Example: ./scripts/deploy.sh G... G... 1000 1735689600 10

CREATOR=${1:?Usage: $0 <creator> <token> <goal> <deadline> <min_contribution>}
TOKEN=${2:?missing token}
GOAL=${3:?missing goal}
DEADLINE=${4:?missing deadline}
MIN_CONTRIBUTION=${5:-1}
NETWORK="testnet"

# Determine the contract output path based on workspace structure
CONTRACT_WASM="target/wasm32-unknown-unknown/release/crowdfund.wasm"

echo "Building WASM..."
cargo build --target wasm32-unknown-unknown --release

echo "Deploying contract to $NETWORK..."
CONTRACT_ID=$(soroban contract deploy \
  --wasm "$CONTRACT_WASM" \
  --network "$NETWORK" \
  --source "$CREATOR")

echo "Contract deployed: $CONTRACT_ID"

echo "Initializing campaign..."
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --network "$NETWORK" \
  --source "$CREATOR" \
  -- \
  initialize \
  --creator "$CREATOR" \
  --token "$TOKEN" \
  --goal "$GOAL" \
  --deadline "$DEADLINE" \
  --min_contribution "$MIN_CONTRIBUTION"

echo "Campaign initialized successfully."
echo "Contract ID: $CONTRACT_ID"
echo "Save this Contract ID for interacting with the campaign."
