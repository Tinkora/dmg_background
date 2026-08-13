# dmg_background

这是一个在浏览器本地运行的 macOS DMG 背景资产编辑器，同时输出带版本的
Finder 布局清单。它生成的是供独立 macOS 打包器消费的设计输入，不会创建、
签名或公证 `.dmg`。

[English](README.md)

[![在 Ko-fi 上支持 Tinkora](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/tinkora)

> **预发布成熟度：** `v0.1.0` 是首个公开的版本化 Release。由于尚无非维护者的实际
> 使用记录，因此这并不代表稳定版本承诺。

[打开浏览器编辑器](https://tinkora.github.io/dmg_background/)

[下载 v0.1.0 及验证资产](https://github.com/Tinkora/dmg_background/releases/tag/v0.1.0)

## 为什么需要它

macOS 应用维护者经常需要反复调整 Finder 窗口、背景图、应用图标和 Applications
别名的位置。视觉设计适合放在浏览器中完成，最终磁盘映像仍需要 macOS 系统工具和
发布凭据。`dmg_background` 将这两个责任明确分开。

## 输出内容

编辑器下载一个只包含以下五个成员的 ZIP：

```text
.background/background.png
.background/background@2x.png
preview.png
dmg_layout.json
README.txt
```

`.background/` 下的两个文件包含背景素材、文字和方向箭头，但不会绘制模拟的 Finder
图标。`preview.png` 会显示模拟图标，便于人工审查。`dmg_layout.json` 使用 Finder
内容区逻辑坐标，供独立打包器读取。

兼容性规则见[输出契约](docs/CONTRACT.zh-CN.md)，明确的非目标见
[产品范围](docs/PRODUCT_SCOPE.zh-CN.md)。

## 能力边界

- **可供人使用：** 浏览器编辑器可以生成并下载资产 ZIP。
- **机器可读：** ZIP 包含带版本的 JSON 布局文档。
- **尚不可由 Agent 调用：** 没有 MCP server、transport、托管 API、身份验证或
  Agent 注册方式。
- **不是 DMG 打包器：** 没有 `hdiutil`、Finder 自动化、签名或公证集成。

仅提供机器可读文件契约，并不等于产品已经可以由 Agent 调用。

## 环境要求

- Rust 1.85 或更新版本
- `wasm32-unknown-unknown` target
- `wasm-pack` 0.15.0
- Node.js 24 或更新版本，用于 Chromium 测试

## 开发

在仓库根目录运行原生和 WASM 检查：

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p dmg_background_web --target wasm32-unknown-unknown --locked
```

构建并在本地运行编辑器：

```bash
wasm-pack build --target web crates/dmg_background_web \
  --out-dir static/pkg --out-name dmg_background_web -- --locked
python3 -m http.server 4173 --directory crates/dmg_background_web/static
```

打开 `http://127.0.0.1:4173/`。

运行真实 WASM 浏览器测试：

```bash
cd crates/dmg_background_web
npm ci
npx --no-install playwright install chromium
npm run test:wasm-smoke:local
```

生成的 `pkg/`、`node_modules/`、Rust `target/` 和浏览器测试产物都已忽略，
不得提交。

## 隐私

用户选择的 PNG 或 JPEG 文件以及生成的 ZIP 字节只保存在浏览器内存中，直到用户主动
下载。应用没有上传路径、分析脚本、Cookie、远程 API、持久化或外部字体依赖。
安装依赖时可能访问当前配置的 Cargo 和 npm registry。

输入与资源边界见[安全策略](SECURITY.zh-CN.md)。

## 参与贡献

提交 Pull Request 前请阅读[贡献指南](CONTRIBUTING.zh-CN.md)。公开提交的主题和正文
使用英文 Conventional Commits。

## 许可证

MIT，详见 [LICENSE](LICENSE)。
