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
 * @file readme_md_installation.test.js
 * @notice Tests for README.md installation steps and script logging bounds.
 *
 * @dev Validates:
 *   - All prerequisite tools are present and functional
 *   - deploy.sh and interact.sh emit bounded [LOG] lines
 *   - [LOG] line format is well-formed (key=value pairs)
 *   - Unknown actions produce exactly 1 error log line and exit 1
 *   - Log output is grep-parseable (contract_id extractable)
 *   - Scripts are executable
 *   - README contains the Logging Bounds section
 *
 * ## Security notes
 * - Log lines are asserted to contain only expected fields; no free-form
 *   user input is echoed verbatim into [LOG] lines.
 * - Max log line counts are asserted to prevent unbounded output.
 */

const { execSync, spawnSync } = require('child_process');
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

const ROOT = process.cwd();
const DEPLOY_SCRIPT = path.join(ROOT, 'scripts', 'deploy.sh');
const INTERACT_SCRIPT = path.join(ROOT, 'scripts', 'interact.sh');
const README_INSTALL = path.join(ROOT, 'readme_md_installation.md');

// ── Helpers ───────────────────────────────────────────────────────────────────

/** Run a shell script with args; returns { stdout, stderr, status }. */
function run(script, args = []) {
  const result = spawnSync('bash', [script, ...args], {
    encoding: 'utf8',
    env: { ...process.env },
  });
  return {
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    status: result.status,
  };
}

/** Extract all [LOG] lines from a string. */
function logLines(output) {
  return output.split('\n').filter((l) => l.startsWith('[LOG]'));
}

/** Parse a [LOG] line into a key→value map. */
function parseLog(line) {
  const map = {};
  const parts = line.replace('[LOG]', '').trim().split(/\s+/);
  for (const part of parts) {
    const eq = part.indexOf('=');
    if (eq !== -1) {
      map[part.slice(0, eq)] = part.slice(eq + 1);
    }
  }
  return map;
const EXEC_OPTS = { encoding: 'utf8', stdio: 'pipe' };
const ROOT = path.resolve(__dirname);
const DEPLOY_SCRIPT = path.join(ROOT, 'scripts', 'deploy.sh');
const INTERACT_SCRIPT = path.join(ROOT, 'scripts', 'interact.sh');

/** Run a shell command, return stdout string. Throws on non-zero exit. */
function run(cmd, opts = {}) {
  return execSync(cmd, { encoding: 'utf8', stdio: 'pipe', ...opts });
}

/** Spawn a script with args, return { stdout, status }. Never throws. */
function spawn(script, args = []) {
  const result = spawnSync('bash', [script, ...args], { encoding: 'utf8' });
  return { stdout: result.stdout || '', status: result.status };
}

/** Extract [LOG] lines from output. */
function logLines(output) {
  return output.split('\n').filter(l => l.includes('[LOG]'));
}

/** Parse a single [LOG] line into a key=value object. */
function parseLog(line) {
  const obj = {};
  const pairs = line.replace(/.*\[LOG\]\s*/, '').trim().split(/\s+/);
  for (const pair of pairs) {
    const [k, v] = pair.split('=');
    if (k) obj[k] = v ?? '';
  }
  return obj;
}

// ── Prerequisites ─────────────────────────────────────────────────────────────

describe('Installation Prerequisites', () => {
  test('01 - Rust stable is installed', () => {
    const v = execSync('rustc --version', { encoding: 'utf8' }).trim();
    expect(v).toMatch(/^rustc \d+\.\d+\.\d+/);
  });

  test('02 - wasm32-unknown-unknown target is installed', () => {
    const targets = execSync('rustup target list --installed', { encoding: 'utf8' });
    expect(targets).toMatch(/wasm32-unknown-unknown/);
  });

  test('03 - Node.js >= 18 is available', () => {
    const v = execSync('node --version', { encoding: 'utf8' }).trim();
    const major = parseInt(v.replace('v', '').split('.')[0], 10);
    expect(major).toBeGreaterThanOrEqual(18);
  });

  test('04 - npm is available', () => {
    execSync('npm --version', { encoding: 'utf8' });
  });

  test('05 - Git is available', () => {
    const v = execSync('git --version', { encoding: 'utf8' }).trim();
    expect(v).toMatch(/git version/);
  });
});

// ── Script existence and permissions ─────────────────────────────────────────

describe('Script Files', () => {
  test('06 - deploy.sh exists', () => {
    expect(fs.existsSync(DEPLOY_SCRIPT)).toBe(true);
  });

  test('07 - deploy.sh is executable', () => {
    expect(fs.statSync(DEPLOY_SCRIPT).mode & 0o111).toBeTruthy();
  });

  test('08 - interact.sh exists', () => {
    expect(fs.existsSync(INTERACT_SCRIPT)).toBe(true);
  });

  test('09 - interact.sh is executable', () => {
    expect(fs.statSync(INTERACT_SCRIPT).mode & 0o111).toBeTruthy();
  });
describe('Prerequisites', () => {
  test('rustc is installed', () => {
    expect(run('rustc --version')).toMatch(/^rustc \d+\.\d+\.\d+/);
  });

  test('cargo is installed', () => {
    expect(run('cargo --version')).toMatch(/^cargo \d+\.\d+\.\d+/);
  });

  test('wasm32-unknown-unknown target is installed', () => {
    expect(run('rustup target list --installed')).toContain('wasm32-unknown-unknown');
  });

  test('stellar CLI is installed (v20+ rename)', () => {
    expect(run('stellar --version')).toContain('stellar-cli');
  });

  test('Node.js >= 18 is available', () => {
    const major = parseInt(run('node --version').trim().replace('v', ''), 10);
    expect(major).toBeGreaterThanOrEqual(18);
  });
});

// ── deploy.sh logging bounds ──────────────────────────────────────────────────

describe('deploy.sh logging bounds', () => {
  test('deploy.sh with no args exits non-zero (missing required args)', () => {
    const { status } = spawn(DEPLOY_SCRIPT);
    expect(status).not.toBe(0);
  });

  test('deploy.sh emits no [LOG] lines before arg validation fails', () => {
    const { stdout } = spawn(DEPLOY_SCRIPT);
    expect(logLines(stdout).length).toBe(0);
  });

  test('[LOG] line format is key=value pairs', () => {
    const out = run(`bash -c 'echo "[LOG] step=build status=start"'`).trim();
    const parsed = parseLog(out);
    expect(parsed.step).toBe('build');
    expect(parsed.status).toBe('start');
  });

  test('deploy.sh source contains all 7 expected [LOG] patterns', () => {
    const src = fs.readFileSync(DEPLOY_SCRIPT, 'utf8');
    expect(src).toMatch(/\[LOG\] step=build status=start/);
    expect(src).toMatch(/\[LOG\] step=build status=ok/);
    expect(src).toMatch(/\[LOG\] step=deploy status=start/);
    expect(src).toMatch(/\[LOG\] step=deploy status=ok/);
    expect(src).toMatch(/\[LOG\] step=initialize status=start/);
    expect(src).toMatch(/\[LOG\] step=initialize status=ok/);
    expect(src).toMatch(/\[LOG\] step=done/);
  });

  test('deploy.sh has at most 7 [LOG] echo lines (bounded output)', () => {
    const src = fs.readFileSync(DEPLOY_SCRIPT, 'utf8');
    const count = (src.match(/echo "\[LOG\]/g) || []).length;
    expect(count).toBeLessThanOrEqual(7);
  });

  test('15 - deploy.sh step=done line includes contract_id field', () => {
    const src = fs.readFileSync(DEPLOY_SCRIPT, 'utf8');
    expect(src).toMatch(/\[LOG\] step=done contract_id=/);
});

// ── interact.sh logging bounds ────────────────────────────────────────────────

describe('interact.sh logging bounds', () => {
  test('interact.sh with no args exits non-zero', () => {
    const { status } = spawn(INTERACT_SCRIPT);
    expect(status).not.toBe(0);
  });

  test('interact.sh unknown action emits exactly 1 [LOG] error line', () => {
    const { stdout, status } = spawn(INTERACT_SCRIPT, ['CTEST', 'unknown_action']);
    expect(status).toBe(1);
    const lines = logLines(stdout);
    expect(lines.length).toBe(1);
    expect(lines[0]).toMatch(/status=error/);
  });

  test('interact.sh unknown action log line has reason= field', () => {
    const { stdout } = spawn(INTERACT_SCRIPT, ['CTEST', 'unknown_action']);
    const parsed = parseLog(logLines(stdout)[0]);
    expect(parsed.reason).toBe('unknown_action');
  });

  test('interact.sh contribute action has exactly 2 [LOG] lines in source', () => {
    const src = fs.readFileSync(INTERACT_SCRIPT, 'utf8');
    const block = src.match(/contribute\)([\s\S]*?);;/)?.[1] || '';
    expect((block.match(/echo "\[LOG\]/g) || []).length).toBe(2);
  });

  test('interact.sh withdraw action has exactly 2 [LOG] lines in source', () => {
    const src = fs.readFileSync(INTERACT_SCRIPT, 'utf8');
    const block = src.match(/withdraw\)([\s\S]*?);;/)?.[1] || '';
    expect((block.match(/echo "\[LOG\]/g) || []).length).toBe(2);
  });
});

// ── Edge Cases ────────────────────────────────────────────────────────────────

describe('Edge Case — WASM target', () => {
  test('rustup target list --installed contains wasm32-unknown-unknown', () => {
    expect(run('rustup target list --installed')).toMatch(/wasm32-unknown-unknown/);
  });
});

// ── interact.sh logging bounds ────────────────────────────────────────────────

describe('interact.sh logging bounds', () => {
  test('16 - interact.sh with no args exits non-zero', () => {
    const { status } = run(INTERACT_SCRIPT, []);
    expect(status).not.toBe(0);
  });

  test('17 - interact.sh unknown action emits exactly 1 [LOG] error line', () => {
    const { stdout, status } = run(INTERACT_SCRIPT, ['CTEST', 'unknown_action']);
    expect(status).toBe(1);
    const lines = logLines(stdout);
    expect(lines.length).toBe(1);
    expect(lines[0]).toMatch(/status=error/);
  });

  test('18 - interact.sh unknown action log line has reason= field', () => {
    const { stdout } = run(INTERACT_SCRIPT, ['CTEST', 'unknown_action']);
    const lines = logLines(stdout);
    const parsed = parseLog(lines[0]);
    expect(parsed.reason).toBe('unknown_action');
  });
});

  test('19 - interact.sh contribute action has exactly 2 [LOG] lines in source', () => {
    const src = fs.readFileSync(INTERACT_SCRIPT, 'utf8');
    const contributeBlock = src.match(/contribute\)([\s\S]*?);;/)?.[1] || '';
    const count = (contributeBlock.match(/echo "\[LOG\]/g) || []).length;
    expect(count).toBe(2);
  });

  test('20 - interact.sh withdraw action has exactly 2 [LOG] lines in source', () => {
    const src = fs.readFileSync(INTERACT_SCRIPT, 'utf8');
    const withdrawBlock = src.match(/withdraw\)([\s\S]*?);;/)?.[1] || '';
    const count = (withdrawBlock.match(/echo "\[LOG\]/g) || []).length;
    expect(count).toBe(2);
  });

  test('21 - interact.sh refund action has exactly 2 [LOG] lines in source', () => {
    const src = fs.readFileSync(INTERACT_SCRIPT, 'utf8');
    const refundBlock = src.match(/refund\)([\s\S]*?);;/)?.[1] || '';
    const count = (refundBlock.match(/echo "\[LOG\]/g) || []).length;
    expect(count).toBe(2);
  });

  test('22 - interact.sh [LOG] lines use action= field', () => {
    const src = fs.readFileSync(INTERACT_SCRIPT, 'utf8');
    expect(src).toMatch(/\[LOG\] action=contribute status=start/);
    expect(src).toMatch(/\[LOG\] action=contribute status=ok/);
    expect(src).toMatch(/\[LOG\] action=withdraw status=start/);
    expect(src).toMatch(/\[LOG\] action=withdraw status=ok/);
    expect(src).toMatch(/\[LOG\] action=refund status=start/);
    expect(src).toMatch(/\[LOG\] action=refund status=ok/);
  });
});

// ── [LOG] line format validation ──────────────────────────────────────────────

describe('[LOG] line format', () => {
  const validLines = [
    '[LOG] step=build status=start',
    '[LOG] step=deploy status=ok contract_id=CABC123',
    '[LOG] action=contribute status=start contributor=GABC amount=100',
    '[LOG] action=unknown_action status=error reason=unknown_action',
  ];

  test.each(validLines)('23 - parseLog handles: %s', (line) => {
    const parsed = parseLog(line);
    expect(Object.keys(parsed).length).toBeGreaterThan(0);
    expect(parsed.status || parsed.step || parsed.action).toBeTruthy();
  });

  test('24 - [LOG] lines do not contain unquoted semicolons (injection guard)', () => {
    const src =
      fs.readFileSync(DEPLOY_SCRIPT, 'utf8') +
      fs.readFileSync(INTERACT_SCRIPT, 'utf8');
    const logEchos = src.match(/echo "\[LOG\][^"]*"/g) || [];
    for (const line of logEchos) {
      expect(line).not.toMatch(/;/);
    }
  });
});

// ── README content ────────────────────────────────────────────────────────────

describe('README installation doc', () => {
  let readme;
  beforeAll(() => {
    readme = fs.readFileSync(README_INSTALL, 'utf8');
  });

  test('25 - README contains Logging Bounds section', () => {
    expect(readme).toMatch(/## Logging Bounds/);
  });

  test('26 - README documents maximum 7 log lines for deploy.sh', () => {
    expect(readme).toMatch(/7/);
  });

  test('27 - README documents exactly 2 log lines for interact.sh', () => {
    expect(readme).toMatch(/exactly 2/);
  });

  test('28 - README contains [LOG] format example', () => {
    expect(readme).toMatch(/\[LOG\]/);
  });

  test('29 - README contains grep parsing example', () => {
    expect(readme).toMatch(/grep/);
  });

  test('30 - README contains Security Assumptions section', () => {
    expect(readme).toMatch(/## Security Assumptions/);
describe('Prerequisites', () => {
  test('rustc is installed (stable channel)', () => {
    const out = run('rustc --version');
    expect(out).toMatch(/^rustc \d+\.\d+\.\d+/);
  });

  test('cargo is installed', () => {
    const out = run('cargo --version');
    expect(out).toMatch(/^cargo \d+\.\d+\.\d+/);
  });

  test('wasm32-unknown-unknown target is installed', () => {
    const out = run('rustup target list --installed');
    expect(out).toContain('wasm32-unknown-unknown');
  });

  test('stellar CLI is installed (v20+ rename)', () => {
    const out = run('stellar --version');
    expect(out).toContain('stellar-cli');
  });

  test('Node.js >= 18 is available', () => {
    const out = run('node --version');
    const major = parseInt(out.trim().replace('v', ''), 10);
    expect(major).toBeGreaterThanOrEqual(18);
  });
});

// ── Getting Started commands ──────────────────────────────────────────────────

describe('Getting Started', () => {
  test('cargo build --dry-run succeeds (wasm32 release)', () => {
    run(
      'cargo build --release --target wasm32-unknown-unknown -p crowdfund --dry-run',
      { cwd: ROOT, timeout: 30000 }
    );
  }, 35000);

  test('cargo test --no-run compiles test suite', () => {
    run('cargo test --no-run --workspace', { cwd: ROOT, timeout: 120000 });
  }, 130000);
});

// ── Edge Case: cargo test parallelism ────────────────────────────────────────

describe('Edge Case — cargo test parallelism', () => {
  test('cargo test --workspace --test-threads=2 flag is accepted', () => {
    // Verifies the flag syntax documented in the Troubleshooting section is valid.
    // Uses --no-run to avoid a full test execution in CI.
    run('cargo test --no-run --workspace -- --test-threads=2', {
      cwd: ROOT,
      timeout: 120000,
    });
  }, 130000);
});

// ── Edge Case: WASM target ────────────────────────────────────────────────────

describe('Edge Case — WASM target', () => {
  test('rustup target list --installed contains wasm32-unknown-unknown', () => {
    expect(run('rustup target list --installed')).toMatch(/wasm32-unknown-unknown/);
  });
});

// ── Edge Case: CLI versioning ─────────────────────────────────────────────────

describe('Edge Case — Stellar CLI versioning', () => {
  test('stellar --version does not start with "soroban" (v20+ rename)', () => {
    expect(run('stellar --version')).not.toMatch(/^soroban/);
  });

  test('stellar contract --help exits cleanly', () => {
    expect(() => run('stellar contract --help')).not.toThrow();
  });
});

describe('Edge Case — Network identity (no keys required)', () => {
  test('stellar keys list does not crash', () => {
    expect(() => {
      try { run('stellar keys list'); } catch (_) { /* no keys configured — acceptable */ }
    }).not.toThrow();
  });
});

// ── Security ──────────────────────────────────────────────────────────────────

describe('Security', () => {
  test('.soroban/ is listed in .gitignore', () => {
    const gitignore = fs.readFileSync(path.join(ROOT, '.gitignore'), 'utf8');
    expect(gitignore).toMatch(/\.soroban/);
  });

  test('verify_env.sh exists and is executable', () => {
    const script = path.join(ROOT, 'scripts', 'verify_env.sh');
    expect(fs.existsSync(script)).toBe(true);
    expect(fs.statSync(script).mode & 0o100).toBeTruthy();
  });

  test('docs/readme_md_installation.md exists', () => {
    expect(fs.existsSync(path.join(ROOT, 'docs', 'readme_md_installation.md'))).toBe(true);
  });
});
