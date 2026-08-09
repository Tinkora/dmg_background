# 贡献指南

[English](CONTRIBUTING.md)

## 范围

贡献内容应改进浏览器本地编辑器、Rust 校验、输出契约、测试、无障碍、文档或可复现构建
流程。Cloud Session、MCP transport、最终 DMG 创建、签名和公证需要单独获得设计与
证据计划批准。

## 工作流

1. Fork 仓库，并从 `main` 创建短期分支。
2. 每个提交只包含一个逻辑变更，并使用英文 Conventional Commits。
3. 改变行为前先添加面向结果的测试。
4. 运行相关的原生、WASM、浏览器、文档和依赖检查。
5. 创建 Pull Request，说明问题、范围、用户影响、隐私影响和准确的验证命令。

不要提交生成的 WASM package、`node_modules`、Rust `target`、浏览器测试产物、凭据、
用户文件或私有漏洞信息。

## 前端变更

前端 Pull Request 必须保持可见键盘焦点、语义化标签、状态播报、reduced-motion、触摸
和键盘操作，并保证 375、768、1024 和 1440 px 宽度没有水平溢出。维护者在修改或合并
HTML 与用户可见前端代码前，会执行仓库要求的 `ui-ux-pro-max` 审查流程。

## 检查命令

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p dmg_background_web --target wasm32-unknown-unknown --locked
cd crates/dmg_background_web && npm ci && npm run test:wasm-smoke
```

行为准则和默认 Pull Request 模板继承自 Tinkora 组织仓库。
