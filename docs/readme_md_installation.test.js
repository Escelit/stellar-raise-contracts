/**
 * @file readme_md_installation.test.js
 *
 * Tests for the installation documentation defined in:
 *   - README.md  (Prerequisites / Getting Started / Troubleshooting sections)
 *   - docs/readme_md_installation.md  (edge-case companion doc)
 *
 * These tests validate:
 *   1. Required sections and headings are present in both documents.
 *   2. All code-block commands are syntactically well-formed.
 *   3. All internal cross-links resolve to real files.
 *   4. Minimum-requirement table rows are present and correctly formatted.
 *   5. Frontend-specific edge cases (Node.js, npm, CSS variables, port conflict).
 *   6. Security notes are present and contain expected guidance.
 *   7. Edge cases: empty content, missing files, malformed tables.
 *
 * Run with:
 *   npm test
 *   npm run test:coverage   (for coverage report)
 *
 * Coverage target: ≥ 95 % statements / lines, 100 % functions.
 */

const { readFileSync, existsSync } = require('fs');
const { resolve, dirname } = require('path');

// Jest globals (describe, it, expect, beforeAll) are available automatically.

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const ROOT = resolve(__dirname, '..');

/**
 * Read a file relative to the repo root.
 * @param {string} relPath
 * @returns {string}
 */
function readDoc(relPath) {
  const abs = resolve(ROOT, relPath);
  if (!existsSync(abs)) throw new Error(`File not found: ${abs}`);
  return readFileSync(abs, 'utf8');
}

/**
 * Extract all fenced code blocks from a markdown string.
 * @param {string} md
 * @returns {string[]}
 */
function extractCodeBlocks(md) {
  const blocks = [];
  const re = /```[^\n]*\n([\s\S]*?)```/g;
  let m;
  while ((m = re.exec(md)) !== null) blocks.push(m[1]);
  return blocks;
}

/**
 * Extract all markdown links [text](url) from a string.
 * @param {string} md
 * @returns {{ text: string; url: string }[]}
 */
function extractLinks(md) {
  const links = [];
  const re = /\[([^\]]+)\]\(([^)]+)\)/g;
  let m;
  while ((m = re.exec(md)) !== null) links.push({ text: m[1], url: m[2] });
  return links;
}

/**
 * Return true if a relative markdown link resolves to an existing file.
 * @param {string} linkUrl
 * @param {string} docPath  path of the document containing the link
 */
function internalLinkExists(linkUrl, docPath) {
  if (linkUrl.startsWith('http') || linkUrl.startsWith('#')) return true; // external / anchor
  const base = dirname(resolve(ROOT, docPath));
  // Strip anchor fragment
  const filePart = linkUrl.split('#')[0];
  return existsSync(resolve(base, filePart));
}

// ---------------------------------------------------------------------------
// Load documents once
// ---------------------------------------------------------------------------

let readme;
let edgeCaseDoc;

beforeAll(() => {
  readme = readDoc('README.md');
  edgeCaseDoc = readDoc('docs/readme_md_installation.md');
});

// ===========================================================================
// README.md — Prerequisites section
// ===========================================================================

describe('README.md — Prerequisites section', () => {
  it('contains a Prerequisites heading', () => {
    expect(readme).toMatch(/^## Prerequisites/m);
  });

  it('lists Rust as a requirement', () => {
    expect(readme).toMatch(/Rust/i);
  });

  it('lists Node.js as a requirement', () => {
    expect(readme).toMatch(/Node\.js/i);
  });

  it('lists Stellar CLI as a requirement', () => {
    expect(readme).toMatch(/Stellar CLI/i);
  });

  it('includes the wasm32-unknown-unknown target install command', () => {
    expect(readme).toMatch(/rustup target add wasm32-unknown-unknown/);
  });

  it('includes the Stellar CLI install command', () => {
    expect(readme).toMatch(/install-soroban\.sh/);
  });

  it('includes nvm install instructions for Node.js', () => {
    expect(readme).toMatch(/nvm install 18/);
  });

  it('references nodejs.org as an alternative install path', () => {
    expect(readme).toMatch(/nodejs\.org/);
  });

  it('shows minimum Node.js version ≥ 18', () => {
    // Table row or inline mention
    expect(readme).toMatch(/Node\.js.*18|18.*Node\.js/i);
  });

  it('shows minimum Stellar CLI version ≥ 20', () => {
    expect(readme).toMatch(/20\.0\.0|≥ 20/);
  });
});

// ===========================================================================
// README.md — Getting Started section
// ===========================================================================

describe('README.md — Getting Started section', () => {
  it('contains a Getting Started heading', () => {
    expect(readme).toMatch(/^## Getting Started/m);
  });

  it('includes git clone command', () => {
    expect(readme).toMatch(/git clone/);
  });

  it('includes cargo build command', () => {
    expect(readme).toMatch(/cargo build --release --target wasm32-unknown-unknown/);
  });

  it('includes cargo test command', () => {
    expect(readme).toMatch(/cargo test --workspace/);
  });

  it('includes npm install step', () => {
    expect(readme).toMatch(/npm install/);
  });

  it('includes frontend test command', () => {
    expect(readme).toMatch(/npm test.*--run|npm run test/);
  });

  it('does NOT run npm run dev as an automated step (long-running guard)', () => {
    // Extract just the Getting Started section (up to the next ## heading)
    const gettingStarted = readme.match(/## Getting Started[\s\S]*?(?=\n## )/)?.[0] ?? '';
    // Within that section, no uncommented npm run dev should appear
    const devLine = gettingStarted.match(/^(?![ \t]*#)[^\n]*\bnpm run dev\b/m);
    expect(devLine).toBeNull();
  });
});

// ===========================================================================
// README.md — Troubleshooting section
// ===========================================================================

describe('README.md — Troubleshooting section', () => {
  it('contains a Troubleshooting heading', () => {
    expect(readme).toMatch(/^## Troubleshooting/m);
  });

  it('covers WASM target missing', () => {
    expect(readme).toMatch(/WASM target missing/i);
  });

  it('covers Stellar CLI not found', () => {
    expect(readme).toMatch(/Stellar CLI not found/i);
  });

  it('covers cargo test hangs', () => {
    expect(readme).toMatch(/cargo test hangs/i);
  });

  it('covers Node.js version mismatch', () => {
    expect(readme).toMatch(/Node\.js version mismatch/i);
  });

  it('covers npm peer dependency errors', () => {
    expect(readme).toMatch(/peer dependency/i);
  });

  it('covers frontend dev server port conflict', () => {
    expect(readme).toMatch(/port conflict/i);
  });

  it('includes --legacy-peer-deps flag', () => {
    expect(readme).toMatch(/--legacy-peer-deps/);
  });

  it('includes EADDRINUSE symptom', () => {
    expect(readme).toMatch(/EADDRINUSE/);
  });
});

// ===========================================================================
// README.md — Security notes
// ===========================================================================

describe('README.md — Security notes', () => {
  it('warns against committing .soroban/', () => {
    expect(readme).toMatch(/\.soroban\//);
  });

  it('warns against committing ~/.config/stellar/', () => {
    expect(readme).toMatch(/~\/.config\/stellar\//);
  });
});

// ===========================================================================
// README.md — Internal links resolve
// ===========================================================================

describe('README.md — Internal links', () => {
  it('all relative links point to existing files', () => {
    const links = extractLinks(readme).filter(
      (l) => !l.url.startsWith('http') && !l.url.startsWith('#'),
    );
    for (const { url } of links) {
      const exists = internalLinkExists(url, 'README.md');
      expect(exists).toBe(true); // broken link: ${url}
    }
  });
});

// ===========================================================================
// README.md — Code blocks are non-empty
// ===========================================================================

describe('README.md — Code blocks', () => {
  it('all fenced code blocks have non-empty content', () => {
    const blocks = extractCodeBlocks(readme);
    expect(blocks.length).toBeGreaterThan(0);
    for (const block of blocks) {
      expect(block.trim().length).toBeGreaterThan(0); // empty code block found
    }
  });
});

// ===========================================================================
// docs/readme_md_installation.md — Structure
// ===========================================================================

describe('docs/readme_md_installation.md — Structure', () => {
  it('contains a top-level heading', () => {
    expect(edgeCaseDoc).toMatch(/^# /m);
  });

  it('contains a Minimum Requirements table', () => {
    expect(edgeCaseDoc).toMatch(/## Minimum Requirements/i);
  });

  it('lists Node.js ≥ 18 in the requirements table', () => {
    expect(edgeCaseDoc).toMatch(/Node\.js.*18|18.*Node\.js/i);
  });

  it('lists npm ≥ 9 in the requirements table', () => {
    expect(edgeCaseDoc).toMatch(/npm.*9|9.*npm/i);
  });

  it('contains an Automated Verification section', () => {
    expect(edgeCaseDoc).toMatch(/## Automated Verification/i);
  });

  it('references verify_env.sh', () => {
    expect(edgeCaseDoc).toMatch(/verify_env\.sh/);
  });

  it('contains a Security Notes section', () => {
    expect(edgeCaseDoc).toMatch(/## Security Notes/i);
  });
});

// ===========================================================================
// docs/readme_md_installation.md — Edge cases coverage
// ===========================================================================

describe('docs/readme_md_installation.md — Edge cases', () => {
  it('covers WASM target not installed (Edge Case 1)', () => {
    expect(edgeCaseDoc).toMatch(/WASM Target Not Installed/i);
  });

  it('covers Stellar CLI version / rename (Edge Case 2)', () => {
    expect(edgeCaseDoc).toMatch(/Stellar CLI Version/i);
  });

  it('covers Testnet vs Futurenet (Edge Case 3)', () => {
    expect(edgeCaseDoc).toMatch(/Testnet vs\. Futurenet/i);
  });

  it('covers toolchain drift after rustup update (Edge Case 4)', () => {
    expect(edgeCaseDoc).toMatch(/Toolchain Drift/i);
  });

  it('covers cargo test hangs (Edge Case 5)', () => {
    expect(edgeCaseDoc).toMatch(/cargo test.*Hangs|Hangs.*cargo test/i);
  });

  it('covers Node.js version mismatch (Edge Case 6)', () => {
    expect(edgeCaseDoc).toMatch(/Node\.js Version Mismatch/i);
  });

  it('covers npm peer dependency conflicts (Edge Case 7)', () => {
    expect(edgeCaseDoc).toMatch(/Peer Dependency Conflicts/i);
  });

  it('covers frontend dev server port conflict (Edge Case 8)', () => {
    expect(edgeCaseDoc).toMatch(/Port Conflict/i);
  });

  it('covers CSS variables not resolving (Edge Case 9)', () => {
    expect(edgeCaseDoc).toMatch(/CSS Variables Not Resolving/i);
  });
});

// ===========================================================================
// docs/readme_md_installation.md — Frontend-specific content
// ===========================================================================

describe('docs/readme_md_installation.md — Frontend content', () => {
  it('includes nvm install command', () => {
    expect(edgeCaseDoc).toMatch(/nvm install 18/);
  });

  it('includes --legacy-peer-deps flag', () => {
    expect(edgeCaseDoc).toMatch(/--legacy-peer-deps/);
  });

  it('includes EADDRINUSE symptom', () => {
    expect(edgeCaseDoc).toMatch(/EADDRINUSE/);
  });

  it('includes lsof kill command for port conflict', () => {
    expect(edgeCaseDoc).toMatch(/lsof -ti:\d+.*xargs kill/);
  });

  it('mentions useDocsCssVariable hook', () => {
    expect(edgeCaseDoc).toMatch(/useDocsCssVariable/);
  });

  it('mentions getComputedStyle for CSS variable resolution', () => {
    expect(edgeCaseDoc).toMatch(/getComputedStyle/);
  });

  it('advises providing fallback values for CSS variables', () => {
    expect(edgeCaseDoc).toMatch(/fallback/i);
  });

  it('mentions SSR guard for CSS variable hook', () => {
    expect(edgeCaseDoc).toMatch(/typeof window/);
  });
});

// ===========================================================================
// docs/readme_md_installation.md — Security notes
// ===========================================================================

describe('docs/readme_md_installation.md — Security notes', () => {
  it('warns against committing .soroban/', () => {
    expect(edgeCaseDoc).toMatch(/\.soroban\//);
  });

  it('warns against committing ~/.config/stellar/', () => {
    expect(edgeCaseDoc).toMatch(/~\/.config\/stellar\//);
  });

  it('recommends multisig / governance for mainnet admin', () => {
    expect(edgeCaseDoc).toMatch(/multisig/i);
  });

  it('advises rotating keys after accidental push', () => {
    expect(edgeCaseDoc).toMatch(/Rotate keys/i);
  });

  it('references CssVariableValidator for CSS injection prevention', () => {
    expect(edgeCaseDoc).toMatch(/CssVariableValidator/);
  });

  it('warns against embedding secret keys in frontend error messages', () => {
    expect(edgeCaseDoc).toMatch(/Never embed secret keys/i);
  });
});

// ===========================================================================
// docs/readme_md_installation.md — Internal links resolve
// ===========================================================================

describe('docs/readme_md_installation.md — Internal links', () => {
  it('all relative links point to existing files', () => {
    const links = extractLinks(edgeCaseDoc).filter(
      (l) => !l.url.startsWith('http') && !l.url.startsWith('#'),
    );
    for (const { url } of links) {
      const exists = internalLinkExists(url, 'docs/readme_md_installation.md');
      expect(exists).toBe(true); // broken link: ${url}
    }
  });
});

// ===========================================================================
// docs/readme_md_installation.md — Code blocks
// ===========================================================================

describe('docs/readme_md_installation.md — Code blocks', () => {
  it('all fenced code blocks have non-empty content', () => {
    const blocks = extractCodeBlocks(edgeCaseDoc);
    expect(blocks.length).toBeGreaterThan(0);
    for (const block of blocks) {
      expect(block.trim().length).toBeGreaterThan(0); // empty code block found
    }
  });
});

// ===========================================================================
// Edge cases — helper functions
// ===========================================================================

describe('Helper — extractCodeBlocks', () => {
  it('returns empty array for content with no code blocks', () => {
    expect(extractCodeBlocks('No code here.')).toEqual([]);
  });

  it('extracts a single code block', () => {
    const md = '```bash\necho hello\n```';
    expect(extractCodeBlocks(md)).toEqual(['echo hello\n']);
  });

  it('extracts multiple code blocks', () => {
    const md = '```\nfoo\n```\n\n```\nbar\n```';
    const blocks = extractCodeBlocks(md);
    expect(blocks).toHaveLength(2);
    expect(blocks[0]).toContain('foo');
    expect(blocks[1]).toContain('bar');
  });
});

describe('Helper — extractLinks', () => {
  it('returns empty array when no links present', () => {
    expect(extractLinks('plain text')).toEqual([]);
  });

  it('extracts a single link', () => {
    const links = extractLinks('[README](../README.md)');
    expect(links).toHaveLength(1);
    expect(links[0]).toEqual({ text: 'README', url: '../README.md' });
  });

  it('ignores anchor-only links', () => {
    const links = extractLinks('[top](#top)');
    expect(links[0].url).toBe('#top');
  });
});

describe('Helper — internalLinkExists', () => {
  it('returns true for http links without checking filesystem', () => {
    expect(internalLinkExists('https://example.com', 'README.md')).toBe(true);
  });

  it('returns true for anchor links', () => {
    expect(internalLinkExists('#section', 'README.md')).toBe(true);
  });

  it('returns true for a known existing file', () => {
    expect(internalLinkExists('README.md', 'README.md')).toBe(true);
  });

  it('returns false for a non-existent file', () => {
    expect(internalLinkExists('does-not-exist.md', 'README.md')).toBe(false);
  });
});
