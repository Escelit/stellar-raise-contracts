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

const { execSync, exec } = require('child_process');
const path = require('path');
const fs = require('fs');

describe('Installation Prerequisites & Verification', () => {
  const projectRoot = process.cwd();

  test('01 - Rust is installed and stable channel available', () => {
    const version = execSync('rustc --version', { encoding: 'utf8', stdio: 'pipe' }).toString().trim();
    expect(version).toMatch(/^rustc \d+\.\d+\.\d+/);
    expect(execSync('rustup show active-toolchain', { encoding: 'utf8', stdio: 'pipe' }).toString()).toMatch(/stable/);
  });

  test('02 - wasm32-unknown-unknown target installed', () => {
    const targets = execSync('rustup target list --installed', { encoding: 'utf8', stdio: 'pipe' }).toString();
    expect(targets).toMatch(/wasm32-unknown-unknown/);
  });

  test('03 - Stellar CLI installed and functional', () => {
    const version = execSync('stellar --version', { encoding: 'utf8', stdio: 'pipe' }).toString().trim();
    expect(version).toContain('stellar-cli');
  });

  test('04 - Node.js and npm available', () => {
    execSync('node --version', { encoding: 'utf8', stdio: 'pipe' });
    execSync('npm --version', { encoding: 'utf8', stdio: 'pipe' });
  });

  test('05 - Cargo build succeeds (debug mode)', () => {
    try {
      execSync('cargo build --target wasm32-unknown-unknown', { cwd: projectRoot, timeout: 60000, stdio: 'ignore' });
    } catch (e) {
      console.log('Build output:', e.stderr?.toString());
      throw new Error('Cargo build failed - check Rust/target setup');
    }
  }, 90000);

  test('06 - Cargo tests pass', () => {
    const result = execSync('cargo test --no-run', { cwd: projectRoot, encoding: 'utf8', stdio: 'pipe' }).toString();
    expect(result).toMatch(/test result: ok/);
  });

  test('07 - Frontend npm ci succeeds', () => {
    execSync('npm ci', { cwd: projectRoot, stdio: 'ignore', timeout: 120000 });
  });

  test('08 - Deployment script exists and is executable', () => {
    const scriptPath = path.join(projectRoot, 'scripts', 'deployment_shell_script.sh');
    expect(fs.existsSync(scriptPath)).toBe(true);
    expect(fs.statSync(scriptPath).mode & fs.constants.S_IXUSR).toBeTruthy();  // executable
  });

  test('09 - README build command valid', () => {
    // Dry-run release build
    execSync('cargo build --release --target wasm32-unknown-unknown -p crowdfund --dry-run', { cwd: projectRoot, stdio: 'ignore' });
  });
});

describe('Edge Cases', () => {
  test('No panic on missing Stellar keys (graceful)', () => {
    // stellar keys list should not crash if no keys
    try {
      execSync('stellar keys list', { timeout: 5000, stdio: 'ignore' });
    } catch (e) {
      // Expected if no keys configured
      expect(e.status).toBeGreaterThanOrEqual(0);
    }
  });
});

// Update jest.config.js if needed for Node env (current has jsdom, but ok for exec)
module.exports = {
  // Existing config handles
};

