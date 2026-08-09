# Export Contract

[简体中文](CONTRACT.zh-CN.md)

## Version

`dmg_layout.json` uses Schema 1 and identifies the public schema as:

```text
https://tinkora.github.io/dmg_background/schema/dmg-layout-v1.json
```

The schema validates document shape and scalar ranges. The Rust validator also
enforces cross-field rules such as required item counts, unique identifiers,
geometry bounds, fixed background paths, and the output pixel budget.

## Coordinates

- The origin is the top-left of the Finder content area.
- `x` increases to the right and `y` increases downward.
- `window.width` and `window.height` are logical points and equal the 1x
  background pixel dimensions.
- Item coordinates identify icon centers.
- The 2x output doubles both dimensions and every rendered coordinate.

## ZIP Members

The ZIP contains exactly these paths:

| Path | Purpose |
| --- | --- |
| `.background/background.png` | Formal 1x Finder background |
| `.background/background@2x.png` | Formal 2x Finder background |
| `preview.png` | Human review image with simulated Finder items |
| `dmg_layout.json` | Schema 1 layout document |
| `README.txt` | Short instructions for a downstream packager |

Formal backgrounds never contain simulated Finder icons or labels. A consumer
must not install `preview.png` as the Finder background.

Before writing the archive, the Rust exporter uses the `png` decoder to decode
all expected pixel rows, verify checksums, reject animation, and check dimensions
against the 1x, 2x, or preview layout target. Decoder acceptance follows the
crate's libpng-compatible handling of otherwise decodable PNG streams.

## Compatibility

Schema URI, version, coordinate space, error codes, and ZIP paths are public
compatibility commitments. Readers may ignore unknown JSON fields so future
minor additions remain readable. Writers must emit the current Schema 1 fields
and pass the Rust cross-field validator.

This contract describes design assets only. Consumers remain responsible for
validating untrusted ZIP and JSON input, creating the DMG on macOS, and keeping
signing and notarization credentials outside this project.
