# libline

BORUIX 的**用户态行编辑库**：把输入单元序列变成一行可编辑的文本。

[English](README.en.md)

## 功能

| 能力 | 说明 |
| --- | --- |
| 字符编辑 | 输入、退格、删除、光标左右移动 |
| 行内跳转 | 跳到行首、行尾、按单词跳转 |
| 历史记录 | 上下翻页浏览已输入的行 |
| Tab 补全 | 按候选前缀补全 |
| 回显抑制 | 输入不上屏（供口令输入使用） |

## 使用

作为依赖被引用：

```toml
[dependencies]
libline = { path = "../libline" }
```

调用方可以选择使用全部能力，或只取其中一部分：

| 调用方 | 使用的能力 |
| --- | --- |
| [`shell`](https://github.com/BRX-Boruix/shell) | 全部——历史、补全、光标编辑、重绘 |
| [`login`](https://github.com/BRX-Boruix/login) | 仅回显抑制 |

## 构建

```bash
cargo test      # 在主机上运行编辑核心的测试
```

## 文件结构

```
libline/src/
├── lib.rs      # 模块导出
├── editor.rs   # 编辑核心
└── source.rs   # 输入源与字节流实现
```

## 相关项目

- [`shell`](https://github.com/BRX-Boruix/shell) —— 使用全部编辑能力
- [`login`](https://github.com/BRX-Boruix/login) —— 仅使用回显抑制
- [`libsys`](https://github.com/BRX-Boruix/libsys) —— 提供系统调用封装

## 许可

MIT License，版权归 Yang Borui 所有。详见 [LICENSE](LICENSE)。
