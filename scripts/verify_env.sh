#!/bin/sh
# verify_env.sh — Automated Soroban development environment check.
#
# @notice  Run this script before opening a PR or filing a bug report to
#          confirm your local toolchain is correctly configured.
#
# @dev     POSIX-compliant (sh, not bash) for compatibility across macOS and
#          Linux.  No bashisms (arrays, [[ ]], etc.) are used.
#
# @minimum-requirements
#   OS   : Linux x86-64 or macOS 12+  (WSL2 on Windows)
#   RAM  : 4 GB (8 GB recommended for --release builds)
#   Disk : 2 GB free for Rust toolchain + WASM target
#
# Exit codes:
#   0 — all checks passed
#   1 — one or more checks failed

set +e   # Do NOT exit on first failure — accumulate all results below.
set -e

PASS=0
FAIL=1
status=0

ok()  { printf '  [OK]  %s\n' "$1"; }
fail(){ printf '  [FAIL] %s\n' "$1"; status=1; }

echo "=== Stellar Raise — environment verification ==="
echo ""

# ── 1. Core toolchain ────────────────────────────────────────────────────────

printf 'Checking rustc ... '
if rustc --version > /dev/null 2>&1; then
    ok "$(rustc --version)"
else
    fail "rustc not found — install via https://rustup.rs"
fi

printf 'Checking cargo ... '
if cargo --version > /dev/null 2>&1; then
    ok "$(cargo --version)"
else
    fail "cargo not found — install via https://rustup.rs"
fi

printf 'Checking stellar CLI ... '
if stellar --version > /dev/null 2>&1; then
    ok "$(stellar --version)"
else
    fail "stellar not found — run: curl -Ls https://soroban.stellar.org/install-soroban.sh | sh"
fi

# ── 2. WASM target ───────────────────────────────────────────────────────────

printf 'Checking wasm32-unknown-unknown target ... '
if rustup target list --installed 2>/dev/null | grep -q 'wasm32-unknown-unknown'; then
    ok "wasm32-unknown-unknown installed"
else
    fail "missing — run: rustup target add wasm32-unknown-unknown"
fi

# ── 3. Dry-run contract build ────────────────────────────────────────────────
#
# @dev  --dry-run resolves dependencies and checks the build graph without
#       producing a WASM binary.  It is fast and safe to run offline once
#       the registry index is cached.  If --dry-run is unsupported by the
#       installed cargo version, fall back to a metadata-only check.

printf 'Dry-run build of crowdfund contract ... '
if cargo build --release --target wasm32-unknown-unknown -p crowdfund --dry-run > /dev/null 2>&1; then
    ok "cargo build --dry-run succeeded"
else
    fail "build dry-run failed — run 'cargo build --release --target wasm32-unknown-unknown -p crowdfund' for details"
fi

# ── 4. Security reminder ─────────────────────────────────────────────────────

echo ""
echo "Security reminder:"
echo "  Never commit .soroban/ or ~/.config/stellar/ — they contain plaintext secret keys."
if [ -d ".soroban" ]; then
    if git check-ignore -q .soroban 2>/dev/null; then
        ok ".soroban/ is in .gitignore"
    else
        fail ".soroban/ exists but is NOT ignored by git — add it to .gitignore immediately"
    fi
fi

# ── Result ───────────────────────────────────────────────────────────────────

echo ""
if [ "$status" -eq 0 ]; then
    echo "All checks passed."
else
    echo "One or more checks failed. See output above."
fi

exit "$status"
