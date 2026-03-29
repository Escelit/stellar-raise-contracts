#!/bin/bash
set -e

# Usage: ./scripts/interact.sh <contract_id> <action> [args...]
# Examples:
#   ./scripts/interact.sh C... contribute G... 100
#   ./scripts/interact.sh C... withdraw G...
#   ./scripts/interact.sh C... refund G...
#
# Logging bounds: every action emits exactly 2 [LOG] lines (start + result).
# Maximum log lines emitted per invocation: 2.
#   ./scripts/interact.sh C... withdraw
#   ./scripts/interact.sh C... refund

CONTRACT_ID=${1:?Usage: $0 <contract_id> <action> [args...]}
ACTION=${2:?missing action: contribute | withdraw | refund}
NETWORK="testnet"

case "$ACTION" in
contribute)
  CONTRIBUTOR=${3:?missing contributor}
  AMOUNT=${4:?missing amount}
  echo "[LOG] action=contribute status=start contributor=$CONTRIBUTOR amount=$AMOUNT"
  stellar contract invoke \
  soroban contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --source "$CONTRIBUTOR" \
    -- \
    contribute \
    --contributor "$CONTRIBUTOR" \
    --amount "$AMOUNT"
  echo "[LOG] action=contribute status=ok contributor=$CONTRIBUTOR amount=$AMOUNT"
  ;;
withdraw)
  CREATOR=${3:?missing creator}
  echo "[LOG] action=withdraw status=start creator=$CREATOR"
  stellar contract invoke \
  echo "Contribution of $AMOUNT from $CONTRIBUTOR successful."
  ;;
withdraw)
  CREATOR=${3:?missing creator}
  echo "[LOG] action=withdraw status=start creator=$CREATOR"
  stellar contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --source "$CREATOR" \
    -- \
    withdraw
  echo "[LOG] action=withdraw status=ok creator=$CREATOR"
  ;;
refund)
  CALLER=${3:?missing caller}
  echo "[LOG] action=refund status=start caller=$CALLER"
  stellar contract invoke \
  echo "Withdraw successful."
  ;;
refund)
  CALLER=${3:?missing caller}
  echo "[LOG] action=refund status=start caller=$CALLER"
  stellar contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --source "$CALLER" \
    -- \
    refund_single \
    --contributor "$CALLER"
  echo "[LOG] action=refund status=ok caller=$CALLER"
  ;;
*)
  echo "[LOG] action=$ACTION status=error reason=unknown_action"
    refund
  echo "Refund successful."
  ;;
*)
  echo "[LOG] action=$ACTION status=error reason=unknown_action"
  exit 1
  ;;
esac
