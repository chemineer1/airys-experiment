# Security and privacy review

Reviewed 24 September 2026 before making the repository public.

## Scope and results

- Reviewed application code, browser loader, build scripts, dependency sources,
  deployment permissions, current tracked files, all six locally retained commit
  revisions, and 54 historical file blobs.
- Gitleaks 8.30.1 found no secrets in current or retained Git history. Manual
  checks found no personal filesystem paths or private addresses in source.
  Commit identities use the public GitHub username and GitHub's noreply address.
- The two demonstration GIFs contain app views, no desktop content or comment
  metadata. The equation assets contain dimensions and alpha pixels only.
- GitHub had one branch, no tags, issues, releases, uploaded Actions artifacts,
  wiki, or discussions containing additional material to expose.
- cargo-audit 0.22.2 checked 522 lockfile entries against the current RustSec
  database: no known vulnerabilities. There is one informational maintenance
  advisory, described below.

## Fixes made before publication

- Browser source links open with `noopener,noreferrer`, preventing the linked
  page from retaining access to the simulation tab or receiving its referrer.
- Rust source paths are remapped during browser builds. Stripping debug info
  alone left local filesystem paths in panic messages; generated app files are
  now checked for these paths before upload and after deployment.
- Pages receives only `index.html`, `airy.js`, and `airy_bg.wasm`. Local captures,
  build caches, tools, and unrelated output files are not deployment inputs.
- Actions are pinned to full commit IDs. Checks have read-only repository
  access; only the deploy job has Pages/OIDC permissions. Pull requests cannot
  deploy, and checkout credentials are not persisted.
- Every deployment runs secret and dependency scans, physics tests, formatting,
  browser Clippy checks, and build validation. The live check verifies the
  expected build, asset hashes, and JavaScript/WebAssembly MIME types.

## Remaining maintenance item

[RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html) marks
`ttf-parser 0.25.1` as unmaintained; it does not identify a vulnerability or a
patched release. The dependency chain is `winit → sctk-adwaita → ab_glyph →
owned_ttf_parser → ttf-parser`, used for native Linux window decorations. It is
absent from the `wasm32-unknown-unknown` browser dependency tree. Track the
upstream replacement when updating native dependencies. CI reports maintenance
warnings and fails on known vulnerabilities; no advisories are suppressed.

## Limits

This is a source, artifact, history, and known-advisory review, not a guarantee
against undiscovered flaws. The application has no accounts, uploads, analytics,
tracking scripts, or browser persistence. GitHub serves the files and can process
ordinary hosting request logs; external sources receive requests when opened.
