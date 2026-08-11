# dmg_background

A browser-local editor for creating macOS DMG background assets and a versioned
Finder layout manifest. It exports design inputs for a separate macOS packager;
it does not create, sign, or notarize a `.dmg`.

[简体中文](README.zh-CN.md)

> **Pre-release maturity:** `v0.1.0` is the first versioned public release. No
> non-maintainer workflow has been recorded yet, so this is not a stable claim.

[Open the browser editor](https://tinkora.github.io/dmg_background/)

[Download v0.1.0 and verification assets](https://github.com/Tinkora/dmg_background/releases/tag/v0.1.0)

## Why This Exists

macOS application maintainers repeatedly need to align a Finder window,
background artwork, an application icon, and the Applications alias. The
visual work is easier in a browser, while final image creation still requires
macOS tools and release credentials. `dmg_background` keeps those concerns
separate.

## Output

The editor downloads one ZIP with exactly five members:

```text
.background/background.png
.background/background@2x.png
preview.png
dmg_layout.json
README.txt
```

The two files under `.background/` contain artwork, text, and the direction
arrow, but never simulated Finder icons. `preview.png` includes simulated icons
for review. `dmg_layout.json` records logical Finder content coordinates for a
separate packager.

See [Contract](docs/CONTRACT.md) for compatibility rules and
[Product scope](docs/PRODUCT_SCOPE.md) for explicit non-goals.

## Capability

- **Human-usable:** the browser editor can create and download the asset ZIP.
- **Machine-readable:** the ZIP contains a versioned JSON layout document.
- **Not Agent-callable:** there is no MCP server, transport, hosted API,
  authentication layer, or Agent registration.
- **Not a DMG packager:** there is no `hdiutil`, Finder automation, signing, or
  notarization integration.

A machine-readable file contract does not by itself make a product
Agent-callable.

## Requirements

- Rust 1.85 or newer
- `wasm32-unknown-unknown` target
- `wasm-pack` 0.15.0
- Node.js 24 or newer for the Chromium suite

## Develop

Run native and WASM checks from the repository root:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p dmg_background_web --target wasm32-unknown-unknown --locked
```

Build and serve the editor locally:

```bash
wasm-pack build --target web crates/dmg_background_web \
  --out-dir static/pkg --out-name dmg_background_web -- --locked
python3 -m http.server 4173 --directory crates/dmg_background_web/static
```

Open `http://127.0.0.1:4173/`.

Run the real WASM browser suite:

```bash
cd crates/dmg_background_web
npm ci
npx --no-install playwright install chromium
npm run test:wasm-smoke:local
```

Generated `pkg/`, `node_modules/`, Rust `target/`, and browser artifacts are
ignored and must not be committed.

## Privacy

Selected PNG or JPEG files and generated ZIP bytes stay in browser memory until
the user downloads them. The application has no upload path, analytics,
cookies, remote API, persistence, or external font dependency. Installing
dependencies can contact the configured Cargo and npm registries.

See [Security](SECURITY.md) for input and resource boundaries.

## Contributing

Read [Contributing](CONTRIBUTING.md) before opening a pull request. Public
commit subjects and bodies use English Conventional Commits.

## License

MIT. See [LICENSE](LICENSE).
