# Repository Guide for AI Agents

## Project Overview

dmg_background is a browser-first macOS DMG background designer. It provides
visual drag-and-drop editing of DMG installer window layouts and exports 1x/2x
backgrounds plus a layout manifest for downstream macOS DMG packaging tools.

## Architecture

```text
dmg_background/
├── crates/
│   ├── dmg_background_core/       # Layout model, validation, ZIP export contract
│   └── dmg_background_web/        # WASM bridge + HTML editor (Canvas 2D)
├── docs/                           # Public contracts and release evidence
└── .github/                        # Hosted quality and supply-chain gates
```

## Key Files for AI Context

| File | Purpose |
| ------ | ------- |
| `crates/dmg_background_core/src/model.rs` | Layout model and coordinates |
| `crates/dmg_background_core/src/validation.rs` | Cross-field validation |
| `crates/dmg_background_core/src/export.rs` | Five-member ZIP contract |
| `crates/dmg_background_core/schema/dmg_layout.schema.json` | JSON Schema v1 |
| `crates/dmg_background_web/src/lib.rs` | WASM bindings (5 JS exports) |
| `crates/dmg_background_web/static/index.html` | Full-featured Canvas editor |
| `docs/CONTRACT.md` | Public ZIP and JSON compatibility contract |
| `docs/PRODUCT_SCOPE.md` | Verified product scope and non-goals |

## Build & Test Commands

```bash
# Run all tests
cargo test --workspace

# Format check
cargo fmt --all -- --check

# Lint (strict)
cargo clippy --workspace --all-targets -- -D warnings

# WASM compilation check
cargo check -p dmg_background_web --target wasm32-unknown-unknown

# Real WASM browser suite
cd crates/dmg_background_web
npm ci
npm run test:wasm-smoke:local

# Build Web WASM for deployment
wasm-pack build --target web crates/dmg_background_web
```

## Design Principles

1. **Browser-first**: Visual editing and asset generation happen in-browser
   through Canvas 2D and WASM.
2. **No uploads**: Assets and generated ZIP files remain in browser memory
   until the user downloads them.
3. **Background vs. preview**: Formal backgrounds never bake Finder items;
   previews may overlay simulated icons.
4. **Contract-driven**: The browser and downstream macOS packager communicate
   only through the versioned ZIP and JSON Schema.
5. **No implied Agent transport**: A machine-readable layout is not an MCP
   server or an Agent-callable tool.

## Coordinate System (Schema v1)

- Origin: top-left of Finder content area
- `x` increases right, `y` increases down
- `window.width/height`: Finder content area in logical points, equal to the 1x
  background pixel dimensions
- `items[].x/y`: icon center point in logical points
- `icon_size`: icon draw side length in logical points (must be even)
- `texts[].x/y`: text rectangle top-left corner
- 2x output: all geometry values multiplied by two

## ZIP Export Contract (5 Frozen Members)

```text
.background/background.png      # 1x formal background (no Finder items)
.background/background@2x.png   # 2x formal background
preview.png                     # Review preview (may include simulated Finder items)
dmg_layout.json                 # Layout manifest (Schema v1)
README.txt                      # Instructions for downstream packager
```

## Error Codes (Stable Machine-Readable)

| Code | Meaning |
| ---- | ------- |
| `UNSUPPORTED_SCHEMA_URI` | Schema URI mismatch |
| `UNSUPPORTED_SCHEMA_VERSION` | Schema version != 1 |
| `UNSUPPORTED_COORDINATE_SPACE` | Coordinate space mismatch |
| `INVALID_WINDOW_SIZE` | Window outside 320..4096 range |
| `INVALID_ICON_SIZE` | Icon size not even or outside 2..4096 |
| `MISSING_REQUIRED_ITEM` | Missing application or applications_alias |
| `DUPLICATE_REQUIRED_ITEM` | More than one required item type |
| `EMPTY_ITEM_ID` | Item has empty ID |
| `DUPLICATE_ITEM_ID` | Duplicate item ID |
| `ITEM_OUT_OF_BOUNDS` | Item icon rectangle exceeds window |
| `INVALID_BACKGROUND_PATHS` | Background paths don't match contract |
| `OUTPUT_DIMENSION_OVERFLOW` | 2x dimensions overflow u32 |
| `OUTPUT_PIXEL_BUDGET_EXCEEDED` | 2x output exceeds the fixed pixel budget |
| `INVALID_PNG_ASSET` | Export input is not a decodable, non-animated PNG |
| `PNG_DIMENSION_MISMATCH` | PNG dimensions do not match the layout |
| `LAYOUT_SERIALIZATION` | JSON serialization failure |
| `ZIP_EXPORT` | ZIP creation failure |

## Commit Language

- Write commit subjects and bodies in English and follow Conventional Commits.
- This repository-level rule overrides any global preference for another
  commit-message language.

## Frontend Design Requirement

- Before creating, modifying, reviewing, or debugging any HTML page or
  user-facing frontend, invoke the `ui-ux-pro-max` skill.
- Run the skill's required `--design-system` search before editing, followed by
  relevant stack and UX searches.
- If `ui-ux-pro-max` is unavailable, stop frontend work and report the missing
  prerequisite.
- Verify the rendered result in a real browser at 375, 768, 1024, and 1440
  pixel widths, including console, keyboard, accessibility, and overflow
  checks.
