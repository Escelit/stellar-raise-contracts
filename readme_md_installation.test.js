/**
 * readme_md_installation.test.js
 *
 * Verifies that the installation commands documented in README.md and
 * docs/readme_md_installation.md are correct and that supporting scripts
 * conform to their documented logging bounds.
 *
 * @security Tests run locally only. No network calls, no Stellar keys required.
 */

'use strict';

const { execSync, spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const ROOT = path.resolve(__dirname);
const DEPLOY_SCRIPT = path.join(ROOT, 'scripts', 'deploy.sh');
const INTERACT_SCRIPT = path.join(ROOT, 'scripts', 'interact.sh');
const EXEC_OPTS = { encoding: 'utf8', stdio: 'pipe' };

// Use real binary paths — snap wrappers silently return empty output from Node.js
const RUST_BIN = '/home/ajidokwu/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin';
const RUSTUP_BIN = '/snap/rustup/current/bin';
// nvm node may not be on the Jest process PATH; find the active version
const NVM_NODE_BIN = (() => {
  const nvm = process.env.NVM_BIN || '';
  if (nvm) return nvm;
  try {
    const { execSync: es } = require('child_process');
    const p = es('bash -c "source ~/.nvm/nvm.sh 2>/dev/null && which node"',
      { encoding: 'utf8', stdio: 'pipe' }).trim();
    return require('path').dirname(p);
  } catch (_) { return ''; }
})();
const AUGMENTED_PATH = [RUST_BIN, RUSTUP_BIN, NVM_NODE_BIN, '/snap/bin', process.env.PATH || ''].filter(Boolean).join(':');
const AUGMENTED_ENV = { ...process.env, PATH: AUGMENTED_PATH };

/** Run a command and return stdout, or throw with a clear message. */
function run(cmd, opts = {}) {
  return execSync(cmd, { ...EXEC_OPTS, env: AUGMENTED_ENV, ...opts });
}

/** Run a script with args via spawnSync; returns { stdout, stderr, status }. */
function runScript(scriptPath, args = []) {
  const result = spawnSync('bash', [scriptPath, ...args], {
    encoding: 'utf8',
    env: AUGMENTED_ENV,
  });
  return {
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    status: result.status,
  };
}

/** Extract [LOG] lines from output. */
function logLines(output) {
  return (output || '').split('\n').filter(l => l.includes('[LOG]'));
}

/** Parse a single [LOG] key=value line into an object. */
function parseLog(line) {
  const obj = {};
  const matches = (line || '').matchAll(/(\w+)=(\S+)/g);
  for (const [, k, v] of matches) obj[k] = v;
  return obj;
}

/** Returns true if the stellar CLI is available. */
function hasStellar() {
  try {
    run('stellar --version');
    return true;
  } catch (_) {
    return false;
  }
}

const STELLAR_AVAILABLE = hasStellar();

// ── Prerequisites ─────────────────────────────────────────────────────────────

describe('Prerequisites', () => {
  const skipIfNoRust = HAS_RUST ? test : test.skip;
  const skipIfNoRustup = HAS_RUSTUP ? test : test.skip;
  const skipIfNoStellar = HAS_STELLAR ? test : test.skip;

  skipIfNoRust('rustc is installed', () => {
    expect(run('rustc --version')).toMatch(/^rustc \d+\.\d+\.\d+/);
  });

  skipIfNoRust('cargo is installed', () => {
    expect(run('cargo --version')).toMatch(/^cargo \d+\.\d+\.\d+/);
  });

  skipIfNoRustup('wasm32-unknown-unknown target is installed', () => {
    expect(run('rustup target list --installed')).toContain('wasm32-unknown-unknown');
  });

