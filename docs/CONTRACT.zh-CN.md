# 输出契约

[English](CONTRACT.md)

## 版本

`dmg_layout.json` 使用 Schema 1，并通过以下公开地址标识 Schema：

```text
https://tinkora.github.io/dmg_background/schema/dmg-layout-v1.json
```

JSON Schema 验证文档结构和标量范围；Rust validator 另外检查必需项目数量、唯一 ID、
几何边界、固定背景路径以及输出像素预算等跨字段规则。

## 坐标

- 原点位于 Finder 内容区左上角。
- `x` 向右增加，`y` 向下增加。
- `window.width` 和 `window.height` 使用逻辑点，也等于 1x 背景像素尺寸。
- 项目坐标表示图标中心点。
- 2x 输出将尺寸和所有渲染坐标同时放大两倍。

## ZIP 成员

ZIP 只包含以下路径：

| 路径 | 用途 |
| --- | --- |
| `.background/background.png` | 正式 1x Finder 背景 |
| `.background/background@2x.png` | 正式 2x Finder 背景 |
| `preview.png` | 包含模拟 Finder 项目的人工审查图 |
| `dmg_layout.json` | Schema 1 布局文档 |
| `README.txt` | 提供给下游打包器的简短说明 |

正式背景不会包含模拟的 Finder 图标或标签。消费者不得把 `preview.png` 安装为 Finder
背景。

写入归档前，Rust exporter 使用 `png` decoder 解码所有预期像素行、校验 checksum、
拒绝动画并根据 1x、2x 或 preview 目标检查尺寸。对于其他仍可解码的 PNG stream，
接受规则遵循该 crate 与 libpng 兼容的宽松语义。

## 兼容性

Schema URI、版本、坐标空间、错误码和 ZIP 路径都是公开兼容性承诺。读取端可以忽略
未知 JSON 字段，从而兼容未来的小幅扩展；写入端必须输出当前 Schema 1 字段，并通过
Rust 跨字段校验。

本契约只描述设计资产。消费者仍需验证不可信 ZIP 与 JSON 输入、在 macOS 上创建 DMG，
并把签名和公证凭据放在本项目之外。
