#!/usr/bin/env bash
#
# Manual testnet deploy for the deEHR Identity / DID Registry contract.
#
# This is a MANUAL operator script — it is intentionally NOT wired into CI.
# Deploying spends testnet KLV from a funded wallet and writes to a live chain.
#
# Prerequisites:
#   - Klever SDK on PATH (`ksc`, `koperator`) — https://docs.klever.org
#   - A funded Klever testnet wallet, exported as a PEM key file
#
# Usage:
#   KEY_FILE=./walletKey.pem ./interaction/deploy_testnet.sh
#
# Optional environment:
#   KLEVER_NODE  testnet node API endpoint
#                (default: https://node.testnet.klever.finance)
#   KEY_FILE     path to the deployer wallet PEM (default: ./walletKey.pem)
#
set -euo pipefail

CONTRACT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM="${CONTRACT_DIR}/output/deehr-identity-registry.wasm"
NODE="${KLEVER_NODE:-https://node.testnet.klever.finance}"
KEY_FILE="${KEY_FILE:-./walletKey.pem}"

command -v ksc >/dev/null || { echo "error: 'ksc' not on PATH (install the Klever SDK)"; exit 1; }
command -v koperator >/dev/null || { echo "error: 'koperator' not on PATH (install the Klever SDK)"; exit 1; }
[ -f "${KEY_FILE}" ] || { echo "error: wallet key file not found: ${KEY_FILE}"; exit 1; }

# 1. Build a fresh WASM artifact (ABI + imports + .wasm under output/).
echo ">> Building contract ..."
( cd "${CONTRACT_DIR}" && ksc all build )

# 2. Deploy. The contract is UPGRADEABLE per ADR-0002 §9 (owner-only at the VM
#    level); `init` takes no arguments. `--await` waits for inclusion.
echo ">> Deploying to ${NODE} ..."
koperator sc create \
  --wasm "${WASM}" \
  --upgradeable \
  --node "${NODE}" \
  --key-file "${KEY_FILE}" \
  --await --sign

echo ">> Done. Record the deployed contract address in this crate's README."
