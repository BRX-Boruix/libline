# libline

**简体中文** | [English](#english)

BORUIX 的**行编辑库**——把按键序列变成一行可编辑的文本。

它提供你在终端里习以为常的那些能力：光标左右移动、退格删除、上下方向键翻历史、Tab 补全。

---

## 它做什么

当程序需要从用户那里读一整行输入时（比如 shell 提示符），不能只是逐字节读进来——用户期望能
**编辑**这一行：发现打错了要能退格、想改前面的字要能移光标、想重复上一条命令要按上方向键。

`libline` 负责这些。它管一件事：**把输入单元序列变成一行可编辑的文本**。

## 为什么单独成库

在这之前，同一套能力在项目里实现了**两遍**——一处是功能完整的版本（历史、补全、光标编辑、
重绘），另一处是一个朴素版本（只有退格）。

两份实现的直接后果是**修复要改两处，而实际只会改一处**。版本逐渐分叉，行为开始不一致。
`libline` 把它们收敛为**一份**。

顺带解决的另一个问题是**口令回显抑制**：登录时输入的密码必须在屏幕上不显示。这是个容易做错
的点——需要正确处理退格、光标移动等编辑操作与"不回显"之间的交互，而不只是简单地把字符吞掉。

## 输入源是可替换的

这个库只依赖一个很窄的接口：**取下一个输入单元**。背后到底是字节流还是事件流，由调用方决定，
**编辑器本体完全不知情**。

这样的好处是：当输入从字节流切换到事件流时，只需要提供另一个输入源实现，**编辑器的代码一行
都不用改**。

输入单元被抽象成一组语义明确的动作，而不是裸字节：

| 输入单元 | 含义 |
| --- | --- |
| `Char` | 一个可打印字符（已解码，含多字节 UTF-8） |
| `Submit` / `Backspace` / `Delete` | 提交、退格、删除 |
| `Left` / `Right` / `Home` / `End` | 光标移动 |
| `HistoryPrev` / `HistoryNext` | 历史浏览 |
| `Complete` | 触发补全 |
| `Interrupt` | 中断（不提交当前行） |
| `Eof` / `WouldBlock` | 输入结束 / 本次无输入可重试 |

**为什么不直接用字节**：字节流能表达的只是"一个字节"，而事件流能表达的是"按下了哪个键"。两者
的共同点是**一个可解释的输入动作**，抽象到这一层，编辑器就不必关心输入的物理来源。

## 一个关键接口约定

输入源接口里有一个容易被误解的钩子：**补充输入**。它返回"本次是否新增了可用字节"。

这个钩子必不可少，原因是：输入源的字节来自外部（键盘、事件总线），而 `libline` 不该知道怎么
去取。如果"本次没取到"之后只能空转等待，输入就永远不会再来。

> 这不是假想的风险——**这正是曾经的第一个真实缺陷**：调用方只在进入读取前泵了一次输入，
> 队列排空后再没有人去取，表现为"程序在运行但敲键盘毫无反应"。

与之配套的一条纪律：**"本次没有得到任何东西"不等于输入结束**。实现若把它当作结束而退出循环，
就会犯下同类错误。正确做法是让出 CPU 后重试。

## 已实现的能力

| 能力 | 说明 |
| --- | --- |
| 基本编辑 | 插入、退格、删除、光标移动 |
| 行内跳转 | 跳到行首、行尾 |
| 单词操作 | 按词移动与删除 |
| 历史记录 | 上下翻阅，**上限 32 条**（环形语义，最旧的被挤出） |
| Tab 补全 | 基于候选集计算公共前缀 |
| 智能大小写匹配 | 补全时的小写输入可匹配大写候选 |
| 口令模式 | 输入不回显，但仍支持编辑操作 |
| 重绘 | 编辑后正确刷新终端显示 |

## 边界

这个库**有意不做**这些事：

- **不是通用界面框架**——它只处理一行文本
- **不做动态库**
- **不做内核行规程**——它工作在用户态
- **不做挂起**（`^Z`）

范围划得窄是有意的：这一类库很容易膨胀成"什么都管一点"的框架，而这里只需要把一行输出来。
需要更复杂交互的程序应该用别的东西。

## 测试

库内含 **49 个单元测试**，覆盖编辑操作、历史、补全匹配与输入源语义。

逻辑部分可以在没有设备的情况下直接测试——这是把输入抽象成窄接口带来的额外好处。

## 构建

```bash
cargo build --release
cargo test
```

## 文件结构

```
libline/
└── src/
    ├── lib.rs       # 公开接口
    ├── editor.rs    # 编辑器本体
    └── source.rs    # 输入源抽象与实现
```

## 相关项目

- [`libsys`](https://github.com/BRX-Boruix/libsys) —— 用户态系统调用封装
- [`shell`](https://github.com/BRX-Boruix/shell) —— 主要使用者
- [`evsrcdemo`](https://github.com/BRX-Boruix/evsrcdemo) —— 输入源组件的验收程序

## 许可

MIT License，版权归 Yang Borui 所有。详见 [LICENSE](LICENSE)。

---

# English

[简体中文](#libline) | **English**

BORUIX's **line editing library** — it turns a sequence of keystrokes into one editable line of text.

It provides the things you take for granted at a terminal: moving the cursor, backspacing, scrolling
history with the arrow keys, and Tab completion.

---

## What it does

When a program needs to read a whole line from the user (a shell prompt, say), it cannot simply read
bytes one at a time — the user expects to **edit** that line: to backspace after a typo, to move the
cursor to fix an earlier word, to press the up arrow to repeat the last command.

`libline` does that. It handles one thing: **turning a sequence of input items into one editable
line of text**.

## Why it is a separate library

Before this, the same capability existed **twice** in the project — one full-featured version
(history, completion, cursor editing, redraw) and one plain version (backspace only).

The direct consequence of two implementations is that **a fix has to be made in two places, and in
practice gets made in one**. The versions drift and start behaving differently. `libline` collapses
them into **one**.

That also settles a related problem: **suppressing echo for passwords**. A password typed at login
must not appear on screen. It is an easy thing to get wrong, because the interaction between editing
operations (backspace, cursor movement) and "do not echo" must be handled correctly, not merely
swallowed character by character.

## The input source is replaceable

The library depends on just one narrow interface: **fetch the next input item**. Whether the backing
is a byte stream or an event stream is the caller's decision, and **the editor itself is unaware of
it**.

The benefit is that when input moves from a byte stream to an event stream, only another input
source implementation is needed — **not one line of the editor changes**.

Input is abstracted into semantically meaningful actions rather than raw bytes:

| Input item | Meaning |
| --- | --- |
| `Char` | A printable character (already decoded, including multi-byte UTF-8) |
| `Submit` / `Backspace` / `Delete` | Submit, backspace, delete |
| `Left` / `Right` / `Home` / `End` | Cursor movement |
| `HistoryPrev` / `HistoryNext` | History navigation |
| `Complete` | Trigger completion |
| `Interrupt` | Interrupt (does not submit the current line) |
| `Eof` / `WouldBlock` | End of input / nothing available this round, retry |

**Why not just bytes**: a byte stream can only express "one byte", whereas an event stream can express
"which key was pressed". What they share is **an interpretable input action**, and abstracting at
that level frees the editor from caring where input physically comes from.

## One crucial interface contract

The input source interface has a hook that is easy to misread: **refill**. It returns whether any new
bytes became available this round.

That hook is indispensable because the bytes come from outside (a keyboard, an event bus) and
`libline` should not know how to fetch them. If "nothing available this round" could only be met by
spinning, input would never arrive again.

> This is not a hypothetical risk — **it was the first real defect**: the caller pumped input once
> before entering the read, and after the queue drained nobody fetched again. The symptom was "the
> program is running but the keyboard does nothing".

The matching discipline: **"nothing available this round" is not end of input**. An implementation
that treats it as such and exits the loop repeats the same class of mistake. The correct move is to
yield the CPU and retry.

## What is implemented

| Capability | Notes |
| --- | --- |
| Basic editing | Insertion, backspace, deletion, cursor movement |
| In-line jumps | Move to start or end of line |
| Word operations | Move and delete by word |
| History | Scroll up and down, **capped at 32 entries** (ring semantics, oldest dropped) |
| Tab completion | Computes the common prefix over a candidate set |
| Smart case matching | Lower-case input can match upper-case candidates during completion |
| Password mode | Input is not echoed, yet editing still works |
| Redraw | The terminal display is refreshed correctly after edits |

## Boundaries

This library deliberately **does not**:

- **Act as a general UI framework** — it handles one line of text
- **Build as a dynamic library**
- **Implement a kernel line discipline** — it works in user space
- **Support suspending** (`^Z`)

The scope is kept narrow on purpose: libraries of this kind easily balloon into a framework that
does a little of everything, whereas all that is needed here is one line of output. Programs wanting
richer interaction should use something else.

## Testing

The library carries **49 unit tests** covering editing operations, history, completion matching, and
input source semantics.

The logic portion can be tested directly without any device — a further benefit of abstracting input
behind a narrow interface.

## Building

```bash
cargo build --release
cargo test
```

## Layout

```
libline/
└── src/
    ├── lib.rs       # the public interface
    ├── editor.rs    # the editor itself
    └── source.rs    # the input source abstraction and implementations
```

## Related projects

- [`libsys`](https://github.com/BRX-Boruix/libsys) — the user-space syscall wrapper
- [`shell`](https://github.com/BRX-Boruix/shell) — the main consumer
- [`evsrcdemo`](https://github.com/BRX-Boruix/evsrcdemo) — acceptance for the input source component

## License

MIT License, copyright Yang Borui. See [LICENSE](LICENSE).
