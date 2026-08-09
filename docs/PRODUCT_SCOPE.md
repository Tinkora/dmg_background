# Product Scope

[简体中文](PRODUCT_SCOPE.zh-CN.md)

## Problem

macOS application maintainers need a repeatable way to prepare Finder window
background artwork and record item coordinates before a separate packaging
step. Existing image editors do not preserve the machine-readable relationship
between the artwork and Finder items.

## Pre-release Scope

The browser editor:

- creates a valid application-to-Applications layout;
- lets a user choose the Finder content size, volume name, background color,
  optional PNG or JPEG, title, and footer;
- positions the application and Applications alias visually or with numeric
  controls;
- shows separate review and formal-background views;
- exports 1x and 2x backgrounds, a review preview, a versioned layout document,
  and a short packager note in one ZIP;
- performs all user-data processing in browser memory.

## Non-goals

This repository does not:

- create, mount, sign, notarize, or distribute a `.dmg`;
- copy an `.app` or create an Applications alias;
- write Finder metadata;
- provide a Cloudflare Worker, hosted session, artifact upload, or storage;
- provide an MCP server, Agent transport, or Agent authentication;
- accept SVG input or promise identical font rendering across operating
  systems.

## Success Evidence

The pre-release scope is considered implemented only when native Rust, MSRV,
WASM, real Chromium, documentation, dependency, and Pages deployment checks
pass for the same public commit. Promotion beyond Pre-release also requires a
recorded non-maintainer workflow using the exported ZIP with an independent
macOS packager.
