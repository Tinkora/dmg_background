# Maturity

[简体中文](MATURITY.zh-CN.md)

## Current Label

**Pre-release.** The repository has no versioned release and no recorded
non-maintainer workflow. Passing tests or deploying a preview does not by
itself justify Alpha, Beta, or Stable.

## Capability Labels

- Human-usable requires a verified public browser workflow.
- Machine-readable requires a versioned contract and independent parser tests.
- Agent-callable requires a runnable transport, registration instructions,
  authentication boundaries where applicable, and integration tests.

`dmg_background` targets the first two labels. It does not currently meet the
Agent-callable definition.

## Promotion Gates

- **Alpha:** all public commit checks and Pages behavior are verified, and one
  non-maintainer completes an export workflow.
- **Beta:** a versioned release exists, an independent macOS packager consumes
  the ZIP, critical failure paths are tested, and support expectations are
  documented from real use.
- **Stable:** the contract has a demonstrated compatibility period, multiple
  external users, a maintained support window, and no unresolved release gate.
