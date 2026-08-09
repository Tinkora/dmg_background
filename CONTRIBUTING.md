# Contributing

[简体中文](CONTRIBUTING.zh-CN.md)

## Scope

Contributions should improve the browser-local editor, Rust validation, export
contract, tests, accessibility, documentation, or reproducible build pipeline.
Cloud sessions, MCP transports, final DMG creation, signing, and notarization
require a separately approved design and evidence plan.

## Workflow

1. Fork the repository and create a short-lived branch from `main`.
2. Keep one logical change per commit and use English Conventional Commits.
3. Add outcome-focused tests before changing behavior.
4. Run the relevant native, WASM, browser, documentation, and dependency checks.
5. Open a pull request with the problem, scope, user impact, privacy impact, and
   exact verification commands.

Do not commit generated WASM packages, `node_modules`, Rust `target`, browser
artifacts, credentials, user files, or private vulnerability details.

## Frontend Changes

Frontend pull requests must preserve visible keyboard focus, semantic labels,
status announcements, reduced-motion behavior, touch and keyboard access, and
no horizontal overflow at 375, 768, 1024, and 1440 px. The maintainer uses the
repository-required `ui-ux-pro-max` review workflow before modifying or merging
HTML and user-facing frontend code.

## Checks

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p dmg_background_web --target wasm32-unknown-unknown --locked
cd crates/dmg_background_web && npm ci && npm run test:wasm-smoke
```

The Code of Conduct and default pull request template are inherited from the
Tinkora organization repository.
