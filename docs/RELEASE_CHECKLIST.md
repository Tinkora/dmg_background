# Release Checklist

[简体中文](RELEASE_CHECKLIST.zh-CN.md)

A local build or green branch does not authorize a release.

- [ ] The exact release commit is on public `main`.
- [ ] Native Rust, MSRV, WASM, Chromium, documentation, supply-chain, CodeQL,
  and Pages checks pass for that commit.
- [ ] The public schema URL returns the committed Schema 1 document.
- [ ] The hosted editor exports the five-member ZIP without external requests,
  console errors, warnings, or horizontal overflow.
- [ ] The version, `CHANGELOG.md`, tag, and release notes agree.
- [ ] No unsupported Worker, MCP, final-DMG, signing, or notarization claim is
  present.
- [ ] The maintainer reviews the exact tag target and deliberately authorizes
  publication.

Until every item is satisfied, keep the project at Pre-release and do not create
a public version tag or GitHub Release.
