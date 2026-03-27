#!/bin/bash
set -e

# Usage: ./scripts/deploy.sh <creator> <token> <goal> <deadline> <min_contribution>
# Example: ./scripts/deploy.sh G... G... 1000 1735689600 10
#
# Logging bounds: every step emits exactly one [LOG] line to stdout.
# Maximum log lines emitted: 7 (one per bounded step below).

CREATOR=${1:?Usage: $0 <creator> <token> <goal> <deadline> <min_contribution>}
TOKEN=${2:?missing token}
GOAL=${3:?missing goal}
DEADLINE=${4:?missing deadline}
MIN_CONTRIBUTION=${5:-1}
NETWORK="testnet"

CONTRACT_WASM="target/wasm32-unknown-unknown/release/crowdfund.wasm"

echo "[LOG] step=build status=start"
cargo build --target wasm32-unknown-unknown --release
echo "[LOG] step=build status=ok"

echo "[LOG] step=deploy status=start network=$NETWORK"
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$CONTRACT_WASM" \
  --network "$NETWORK" \
  --source "$CREATOR")
echo "[LOG] step=deploy status=ok contract_id=$CONTRACT_ID"

echo "[LOG] step=initialize status=start"
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --network "$NETWORK" \
  --source "$CREATOR" \
  -- \
  initialize \
  --admin "$CREATOR" \
  --creator "$CREATOR" \
  --token "$TOKEN" \
  --goal "$GOAL" \
  --deadline "$DEADLINE" \
  --min_contribution "$MIN_CONTRIBUTION" \
  --platform_config "null" \
  --bonus_goal "null" \
  --bonus_goal_description "null"
echo "[LOG] step=initialize status=ok"

echo "[LOG] step=done contract_id=$CONTRACT_ID"
