# 产品范围

[English](PRODUCT_SCOPE.md)

## 问题

macOS 应用维护者需要在独立打包步骤之前，反复准备 Finder 窗口背景并记录项目坐标。
普通图像编辑器不会保留背景素材和 Finder 项目之间可供机器读取的关系。

## 预发布范围

浏览器编辑器可以：

- 创建有效的“应用拖到 Applications”布局；
- 设置 Finder 内容区尺寸、卷名、背景色、可选 PNG 或 JPEG、标题和页脚；
- 通过画布或数字控件调整应用和 Applications 别名位置；
- 分别显示审查预览与正式背景；
- 在一个 ZIP 中输出 1x/2x 背景、审查预览、带版本的布局文档和简短打包说明；
- 只在浏览器内存中处理用户数据。

## 非目标

本仓库不会：

- 创建、挂载、签名、公证或分发 `.dmg`；
- 复制 `.app` 或创建 Applications 别名；
- 写入 Finder 元数据；
- 提供 Cloudflare Worker、托管 Session、产物上传或存储；
- 提供 MCP server、Agent transport 或 Agent 身份验证；
- 接受 SVG 输入，或承诺不同操作系统上的字体渲染完全一致。

## 成功证据

只有同一个公开提交同时通过原生 Rust、MSRV、WASM、真实 Chromium、文档、依赖和
Pages 部署检查，才能认为预发布范围已经实现。升级到 Pre-release 以上还需要记录一次
非维护者使用独立 macOS 打包器消费导出 ZIP 的真实工作流。
