# 发布检查清单

[English](RELEASE_CHECKLIST.md)

本地构建成功或分支检查为绿色，并不自动授权发布。

- [ ] 准确的发布提交已经位于公开 `main`。
- [ ] 该提交的原生 Rust、MSRV、WASM、Chromium、文档、供应链、CodeQL 和 Pages
  检查全部通过。
- [ ] 公开 Schema URL 返回仓库提交的 Schema 1 文档。
- [ ] 托管编辑器可以在没有外部请求、控制台错误、警告或水平溢出的情况下输出五成员 ZIP。
- [ ] 版本号、`CHANGELOG.md`、tag 和 Release Notes 一致。
- [ ] 不存在未支持的 Worker、MCP、最终 DMG、签名或公证声明。
- [ ] 维护者审查准确的 tag 目标，并明确授权发布。

只要有一项尚未满足，就应继续保持 Pre-release，不创建公开版本 tag 或 GitHub Release。
