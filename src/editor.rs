//! 编辑器核心：**纯逻辑**，不触碰 syscall / 终端。
//!
//! # 为何拆成两层
//!
//! 行编辑有两类关注点，而它们的**可测性天差地别**：
//!
//! 1. **编辑语义**（光标移动后缓冲区长什么样、退格删哪个字符、历史怎么翻页）
//!    —— **纯函数**，可宿主测试；
//! 2. **渲染**（发什么字节给终端、何时重绘）
//!    —— 需要终端，只能在 QEMU 里看。
//!
//! 本模块只做第 1 类：把 [`InputItem`] 序列变成缓冲区状态变迁。
//! 渲染通过 [`EditorHost`] trait 由调用方提供，故编辑器本身可在宿主上
//! 用**假 host** 驱动，断言缓冲区与渲染调用序列。

use alloc::vec::Vec;

use crate::source::{InputItem, InputSource};

/// 编辑器对宿主（调用方）的要求。
///
/// **为何是 trait 而非直接调 `libsys::write`**：直接写死会让编辑器
/// 在宿主上**根本跑不起来**（没有 STDIN/STDOUT），于是所有编辑语义
/// 都只能在 QEMU 里肉眼验证。抽成 trait 后，编辑语义可被逐条断言，
/// 而 host 实现只需记录调用。
pub trait EditorHost {
    /// 向终端写出原始字节。
    fn write(&mut self, bytes: &[u8]);

    /// 重绘当前提示符 + 缓冲区，并把光标放在 `cursor` 处。
    ///
    /// **为何由 host 实现而不由库实现**：提示符内容（当前目录、
    /// 用户名）是**调用方的**业务，库不应知道。库只告诉 host：
    /// 「缓冲区是这个，光标在这里，你自己把它画对」。
    fn redraw(&mut self, buffer: &[u8], cursor: usize);

    /// 取补全候选（用于 Tab）。`word` 是当前待补全的词，
    /// `is_command` 补全位置是否为命令位（否则为环境变量）。
    fn completions(&mut self, word: &[u8], is_command: bool) -> Vec<Vec<u8>>;
}

/// 编辑器在一次输入后的决定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditAction {
    /// 继续等下一个输入单元。
    Continue,
    /// 该行已提交：返回最终文本。
    Submitted(Vec<u8>),
    /// 被中断（`^C`）：本行作废，不提交。
    Interrupted,
    /// 输入已结束（EOF）：返回已收集的内容。
    Eof(Vec<u8>),
}

/// 历史环游状态。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryNav {
    /// `None` = 未进入历史游犯；`Some(o)` = 距最新条的偏移（0 = 最新）。
    pub offset: Option<usize>,
    /// 进入历史前的草稿（用于向下退出历史时恢复）。
    pub draft: Vec<u8>,
}

/// 行编辑器：持有缓冲区、光标与历史环，**不持有任何内核句柄**。
#[derive(Clone, Debug, Default)]
pub struct Editor {
    /// 当前行缓冲区。
    pub buffer: Vec<u8>,
    /// 光标位置（字节偏移，0..=buffer.len()）。
    pub cursor: usize,
    /// 历史条目（旧→新）。
    pub history: Vec<Vec<u8>>,
    /// 历史游犯状态。
    pub nav: HistoryNav,
    /// 是否抑制回显（口令输入：ADR-046 决策 3）。
    pub echo_suppressed: bool,
    /// 是否已被 `^C` 中断。
    pub interrupted: bool,
    /// 能力集：哪些高级能力对本实例开放。
    ///
    /// **为何是运行时标志而非类型参数**：`login` 与 `shell` 共用同一个
    /// `Editor`，区别只在「调用方用不用」。若用泛型参数表达，`login` 就得
    /// 拟一个类型出来，增加它本不该有的构造负担。运行时标志同样能
    /// **可执行地**拒绝越权：见 `apply` 里的门禁。
    pub caps: EditorCaps,
    /// `^C` 要投递 `SIGINT` 到的**目标 pid**（ADR-046 §1.2 决策 2）。
    ///
    /// `None` = 不投递（例如 `login`，或 shell 当前没有前台子进程）。
    ///
    /// **为何由调用方设置而不是库自己去查**：目标语义已裁定为
    /// 「shell 自己 spawn 的那个 `child` pid」——那是**调用方才知道**的事实。
    /// 库若自己去查（例如问内核「谁是前台」），既越权也不成立：
    /// 本内核**没有**前台进程组这一概念（ADR-043 决策 1 已拒绝 POSIX 会话/
    /// 进程组），所以「前台」只存在于 shell 的记账里。
    pub interrupt_target: Option<u64>,
}

/// 编辑器能力集（ADR-046 §4 条 3 的裁剪落点）。
///
/// 默认全开（`shell` 用）；`login` 只取回显控制，历史与补全在该实例上不可达。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorCaps {
    /// 历史上翻/下翻是否可用。
    pub history: bool,
    /// Tab 补全是否可用。
    pub completion: bool,
}

impl Default for EditorCaps {
    fn default() -> Self {
        Self::full()
    }
}

impl EditorCaps {
    /// 全开（交互 shell 用）。
    pub const fn full() -> Self {
        Self { history: true, completion: true }
    }
    /// 裁剪到最小：只有逐字读入 + 退格 + 回显控制（登录用）。
    pub const fn plain() -> Self {
        Self { history: false, completion: false }
    }
}
impl Editor {
    /// 新编辑器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否抑制回显。
    pub fn with_echo_suppressed(mut self, on: bool) -> Self {
        self.echo_suppressed = on;
        self
    }
    /// 设置 `^C` 的投递目标（见 [`Editor::interrupt_target`]）。
    ///
    /// 返回 `self` 以便链式构造；shell 在前台子进程存活期间设为 `Some(child)`，
    /// 子进程结束后设回 `None`。
    pub fn with_interrupt_target(mut self, pid: Option<u64>) -> Self {
        self.interrupt_target = pid;
        self
    }

    /// 处理一个输入单元，返回编辑器的决定。
    ///
    /// 这是**编辑语义的单点**：所有对缓冲区/光标/历史的改动都经过它，
    /// 因此宿主单测只需驱动它就能锁定行为。
    pub fn apply(&mut self, item: InputItem, host: &mut impl EditorHost) -> EditAction {
        match item {
            InputItem::WouldBlock => EditAction::Continue,
            InputItem::Submit => {
                let line = core::mem::take(&mut self.buffer);
                self.cursor = 0;
                self.nav = HistoryNav::default();
                self.push_history(&line);
                EditAction::Submitted(line)
            }
            InputItem::Interrupt => {
                self.interrupted = true;
                self.buffer.clear();
                self.cursor = 0;
                self.nav = HistoryNav::default();
                // ADR-046 §1.2 决策 2：`^C` 的**产生端在库内**——识别到控制字节
                // 就投递 `SIGINT`，而不是把「要不要发信号」留给每个调用方各自
                // 记得做（那正是 S28 要避免的重复）。
                //
                // **目标由调用方给定**（`interrupt_target`），库不猜：本内核没有
                // 前台进程组，"谁算前台"只在 shell 的记账里（见字段文档）。
                //
                // **`None` 时静默不投递**：没有前台子进程可打断，`^C` 的语义就是
                // 单纯作废当前输入行——这不是失败，故不报错。
                //
                // **投递失败也不改变 `Interrupted` 结论**：`kill` 可能因目标已退出
                // 而返回 NotFound（典型的竞态：子进程恰在按键前结束）。用户要的是
                // 「回到干净提示符」，已达到；把错误弹出来只会是噪音。故如实忽略。
                if let Some(pid) = self.interrupt_target {
                    let _ = libsys::signal::raise(pid as u64, libsys::signal::SIGINT);
                }
                EditAction::Interrupted
            }
            InputItem::Eof => EditAction::Eof(core::mem::take(&mut self.buffer)),
            InputItem::Char(c) => {
                self.insert_char(c);
                EditAction::Continue
            }
            InputItem::Backspace => {
                if self.cursor > 0 {
                    self.buffer.remove(self.cursor - 1);
                    self.cursor -= 1;
                }
                EditAction::Continue
            }
            InputItem::Delete => {
                if self.cursor < self.buffer.len() {
                    self.buffer.remove(self.cursor);
                }
                EditAction::Continue
            }
            InputItem::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                EditAction::Continue
            }
            InputItem::Right => {
                if self.cursor < self.buffer.len() {
                    self.cursor += 1;
                }
                EditAction::Continue
            }
            InputItem::Home => {
                self.cursor = 0;
                EditAction::Continue
            }
            InputItem::End => {
                self.cursor = self.buffer.len();
                EditAction::Continue
            }
            // Tab: capability off -> silently ignore (no insert, no candidate request).
            // Silent (not an error) because Tab at a login prompt is a stray keystroke;
            // what matters is that it must NOT complete. The gate lives here, so the
            // host's `completions` is never called for a plain editor.
            InputItem::Complete => {
                if self.caps.completion {
                    self.complete(host);
                }
                EditAction::Continue
            }
            // Up/Down: capability off -> ignore, so `login` cannot reach history.
            InputItem::HistoryPrev => {
                if self.caps.history {
                    self.history_prev();
                }
                EditAction::Continue
            }
            InputItem::HistoryNext => {
                if self.caps.history {
                    self.history_next();
                }
                EditAction::Continue
            }
        }
    }

    /// 在光标处插入一个字符。
    pub fn insert_char(&mut self, c: char) {
        let mut tmp = [0u8; 4];
        let bytes = c.encode_utf8(&mut tmp).as_bytes();
        for (i, b) in bytes.iter().enumerate() {
            self.buffer.insert(self.cursor + i, *b);
        }
        self.cursor += bytes.len();
    }

    /// 历史去重（连续重复不记；空行不记）。
    pub fn push_history(&mut self, line: &[u8]) {
        if line.is_empty() {
            return;
        }
        if let Some(last) = self.history.last() {
            if last.as_slice() == line {
                return;
            }
        }
        self.history.push(line.to_vec());
    }

    /// 历史上一条（↑）。已到最旧则停住（不绕回）。
    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        match self.nav.offset {
            None => {
                self.nav.draft = self.buffer.clone();
                self.nav.offset = Some(0);
            }
            Some(o) => {
                if o + 1 < self.history.len() {
                    self.nav.offset = Some(o + 1);
                }
            }
        }
        let o = self.nav.offset.unwrap_or(0);
        let idx = self.history.len() - 1 - o;
        self.buffer = self.history[idx].clone();
        self.cursor = self.buffer.len();
    }

    /// 历史下一条（↓）。退出历史时恢复草稿。
    pub fn history_next(&mut self) {
        let Some(o) = self.nav.offset else { return };
        if o > 0 {
            self.nav.offset = Some(o - 1);
            let idx = self.history.len() - 1 - (o - 1);
            self.buffer = self.history[idx].clone();
        } else {
            self.nav.offset = None;
            self.buffer = self.nav.draft.clone();
        }
        self.cursor = self.buffer.len();
    }

    /// Tab 补全：取当前词的候选，补到最长公共前缀；
    /// 唯一候选则直接补完（命令位追一个空格）。
    pub fn complete(&mut self, host: &mut impl EditorHost) {
        let ws = word_start(&self.buffer);
        let word = self.buffer[ws..].to_vec();
        let (prefix, with_dollar, is_cmd) = if word.first() == Some(&b'$') {
            (&word[1..], true, false)
        } else if ws == 0 {
            (&word[..], false, true)
        } else {
            (&word[..], false, false)
        };

        let matches = host.completions(prefix, is_cmd);
        if matches.is_empty() {
            return;
        }

        let mut rep: Vec<u8> = Vec::new();
        if with_dollar {
            rep.push(b'$');
        }
        if matches.len() == 1 {
            rep.extend_from_slice(&matches[0]);
            if is_cmd {
                rep.push(b' ');
            }
        } else {
            let lcp = longest_common_prefix(&matches);
            rep.extend_from_slice(&matches[0][..lcp]);
        }
        self.buffer.truncate(ws);
        self.buffer.extend_from_slice(&rep);
        self.cursor = self.buffer.len();
    }
}

/// 当前词的起始偏移（最后一个空白之后）。
pub fn word_start(buf: &[u8]) -> usize {
    let mut ws = 0usize;
    for k in 0..buf.len() {
        if buf[k] == b' ' || buf[k] == b'\t' {
            ws = k + 1;
        }
    }
    ws
}

/// 所有候选的最长公共前缀长度。
pub fn longest_common_prefix(matches: &[Vec<u8>]) -> usize {
    if matches.len() <= 1 {
        return matches.first().map(|m| m.len()).unwrap_or(0);
    }
    let mut lcp = 0usize;
    'outer: while lcp < matches[0].len() {
        let b = matches[0][lcp];
        for m in &matches[1..] {
            if lcp >= m.len() || m[lcp] != b {
                break 'outer;
            }
        }
        lcp += 1;
    }
    lcp
}

/// Smart-Case 智能匹配：前缀全小写则忽略大小写；含大写则严格匹配。
pub fn smart_case_match(prefix: &[u8], candidate: &[u8]) -> bool {
    if candidate.len() < prefix.len() {
        return false;
    }
    let has_uppercase = prefix.iter().any(|b| b.is_ascii_uppercase());
    if has_uppercase {
        &candidate[..prefix.len()] == prefix
    } else {
        candidate[..prefix.len()]
            .iter()
            .zip(prefix.iter())
            .all(|(c, p)| c.to_ascii_lowercase() == *p)
    }
}

/// 一次「朴素读行」的配置（ADR-046 §4 条 3 的裁剪形态定稿）。
///
/// # 为何要有这个类型（而不是让 login 直接用 [`Editor`]）
///
/// ADR-046 §4 条 3 的要求：`login` 刻意保持朴素（S24 单一组件聚焦），
/// 引入 `libline` **不应**让它背上历史/Tab 等无关能力——只取**回显抑制子集**。
///
/// 若 `login` 直接构造 [`Editor`]，它能拿到 `history` / `complete()` 等
/// 全套能力；「不该用」只是**口头约定**，编译器不管。本类型 + [`read_line_plain`]
/// 把约束**变成接口事实**：只暴露 `echo` 一个开关，不给历史、不给补全。
///
/// # 能力边界（明确到项）
///
/// | 能力 | `login` 是否需要 | 本接口是否提供 |
/// |---|---|---|
/// | 逐字符读入 + 退格 | ✅ 需要 | ✅ |
/// | 回显抑制（口令） | ✅ 需要 | ✅ `echo` |
/// | 光标左右 / Home / End | ❌ 不需要 | ⚠️ 见下 |
/// | 历史上下翻 | ❌ 不需要 | ❌ 不提供 |
/// | Tab 补全 | ❌ 不需要 | ❌ 不提供 |
///
/// **诚实说明「光标左右」**：`libline` 的编辑器核心天然支持光标移动，
/// 而 `login` 的输入源又必须把方向键（CSI 序列）解析成某个输入单元。
/// 二者相遇时 [`read_line_plain`] 会**照常处理光标移动**——这比「把方向键
/// 当普通字符插进用户名」更正确，且不给 `login` 增加任何**它自己要写的代码**。
/// 换言之：被裁掉的是**历史与补全**——即需要调用方提供数据与逻辑的那两项，
/// 而非核心自带的编辑动作。这与 §4 条 3 的用意一致：不让 `login` 背上
/// 「维护一份历史列表」「提供补全候选」这类负担。
#[derive(Clone, Copy, Debug, Default)]
pub struct PlainLineOptions {
    /// 是否**抑制**回显。
    ///
    /// `true` 用于读口令：此时**任何**输入都不上屏。
    /// `false` 用于读用户名：边打边显示。
    ///
    /// **命名为何是 `suppress_echo` 而非 `echo`**：初版叫 `echo`，于是
    /// `echo: false` 到底是「不回显」还是「不抑制」需要读两遍才能确定，
    /// 而口令泄露的代价太高。改名后语义单向：取值直接就是
    /// `Editor::echo_suppressed`。
    ///
    /// **该倒置错误当场被两项测试抓出**（`plain_read_with_echo_off_never_redraws`
    /// 与它的对照组）：初版把 `echo: false` 映射成「不抑制」，测试立刻报
    /// `redraw` 被调了 7 次。这正是否决「靠命名约定区分」的理由。
    pub suppress_echo: bool,
}

/// 读一行，**不带历史、不带补全**（`login` 的裁剪接口）。
///
/// 返回值语义与 [`read_line`] 一致，便于调用方统一处理。
///
/// **与 [`read_line`] 的唯一区别**：本函数**不**向调用方索取任何补全候选，
/// 故调用方的 [`EditorHost`] 实现里 `completions` 永远不会被调用
/// （可以如实返回空）。这就是「按需裁剪」的落地形态。
pub fn read_line_plain(
    editor: &mut Editor,
    src: &mut impl InputSource,
    host: &mut impl EditorHost,
    prompt: impl Fn(&mut dyn EditorHost),
    opts: PlainLineOptions,
) -> EditAction {
    editor.echo_suppressed = opts.suppress_echo;
    editor.caps = EditorCaps::plain();
    read_line(editor, src, host, prompt)
}
/// 从输入源读一行（驱动编辑器直到提交/中断/EOF）。
///
/// 这是对外的主循环：**只知道 [`InputSource`] 与 [`EditorHost`]**，
/// 不知道背后是字节还是事件，也不知道终端是什么。
pub fn read_line(
    editor: &mut Editor,
    src: &mut impl InputSource,
    host: &mut impl EditorHost,
    prompt: impl Fn(&mut dyn EditorHost),
) -> EditAction {
    prompt(host);
    loop {
        let item = src.next_item();
        if matches!(item, InputItem::WouldBlock) {
            // 队列空：**请输入源自己去补**（它才知道背后是键盘还是事件总线）。
            // 补到东西就立刻重试；补不到才让出 CPU，避免空转烧满。
            //
            // **此处曾是真实缺陷**：初版无条件 yield 后 continue，而调用方
            // 只在进本函数前泵过一次输入，于是后续按键永远进不来——
            // 用户看到的是「登录后 shell 死了」。见 L-2 验收记录。
            if src.refill() {
                continue;
            }
            let _ = libsys::yield_now();
            continue;
        }
        let before_len = editor.buffer.len();
        let before_cur = editor.cursor;
        let action = editor.apply(item, host);
        match action {
            EditAction::Continue => {
                // 编辑器改了缓冲区/光标就重绘；抑制回显时不重绘。
                if !editor.echo_suppressed
                    && (editor.buffer.len() != before_len || editor.cursor != before_cur)
                {
                    host.redraw(&editor.buffer, editor.cursor);
                }
            }
            other => return other,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::ByteSource;
    use alloc::vec;

    /// 假 host：记录所有调用，不触碰终端。
    #[derive(Default)]
    struct FakeHost {
        writes: Vec<Vec<u8>>,
        redraws: Vec<(Vec<u8>, usize)>,
        completions: Vec<Vec<u8>>,
    }

    impl EditorHost for FakeHost {
        fn write(&mut self, bytes: &[u8]) {
            self.writes.push(bytes.to_vec());
        }
        fn redraw(&mut self, buffer: &[u8], cursor: usize) {
            self.redraws.push((buffer.to_vec(), cursor));
        }
        fn completions(&mut self, _word: &[u8], _is_command: bool) -> Vec<Vec<u8>> {
            self.completions.clone()
        }
    }

    fn drive(e: &mut Editor, h: &mut FakeHost, items: &[InputItem]) -> Vec<EditAction> {
        items.iter().map(|i| e.apply(i.clone(), h)).collect()
    }

    // ---------------- 输入单元解析（字节 → 单元） ----------------

    /// 普通可打印字符 → `Char`；回车→`Submit`；退格→`Backspace`。
    #[test]
    fn ascii_bytes_map_to_items() {
        let mut s = ByteSource::new();
        s.push_bytes(b"ab\r");
        assert_eq!(s.next_item(), InputItem::Char('a'));
        assert_eq!(s.next_item(), InputItem::Char('b'));
        assert_eq!(s.next_item(), InputItem::Submit);
        assert_eq!(s.next_item(), InputItem::WouldBlock);
    }

    /// ANSI CSI 序列（上/下/左/右）必须被译为对应动作，
    /// **不得**漏成字符（否则 ESC 会被当普通字符插入缓冲区）。
    #[test]
    fn ansi_csi_arrows_decode_to_actions() {
        let mut s = ByteSource::new();
        s.push_bytes(b"\x1b[A\x1b[B\x1b[C\x1b[D\x1b[H\x1b[F");
        assert_eq!(s.next_item(), InputItem::HistoryPrev);
        assert_eq!(s.next_item(), InputItem::HistoryNext);
        assert_eq!(s.next_item(), InputItem::Right);
        assert_eq!(s.next_item(), InputItem::Left);
        assert_eq!(s.next_item(), InputItem::Home);
        assert_eq!(s.next_item(), InputItem::End);
    }

    /// `\x1b[3~` 是 Delete；`\x1b[1~`/`\x1b[4~` 是 Home/End。
    #[test]
    fn ansi_csi_tilde_forms() {
        let mut s = ByteSource::new();
        s.push_bytes(b"\x1b[3~\x1b[1~\x1b[4~");
        assert_eq!(s.next_item(), InputItem::Delete);
        assert_eq!(s.next_item(), InputItem::Home);
        assert_eq!(s.next_item(), InputItem::End);
    }

    /// 控制字节：`^C`(0x03) → `Interrupt`；`^D`(0x04) → `Eof`；Tab(0x09) → `Complete`。
    #[test]
    fn control_bytes_map_to_signals() {
        let mut s = ByteSource::new();
        s.push_bytes(&[0x03, 0x04, 0x09]);
        assert_eq!(s.next_item(), InputItem::Interrupt);
        assert_eq!(s.next_item(), InputItem::Eof);
        assert_eq!(s.next_item(), InputItem::Complete);
    }

    /// 多字节 UTF-8（中文）必须被收齐为**单个** `Char`，
    /// 而不是逐字节当作字符（否则缓冲区里会出现碎片）。
    #[test]
    fn multibyte_utf8_assembles_one_char() {
        let mut s = ByteSource::new();
        s.push_bytes("中文".as_bytes());
        assert_eq!(s.next_item(), InputItem::Char('中'));
        assert_eq!(s.next_item(), InputItem::Char('文'));
    }

    /// UTF-8 碎片（跨次 `push_bytes`）也要正确拼接。
    #[test]
    fn split_utf8_across_pushes() {
        let mut s = ByteSource::new();
        let bytes = "中".as_bytes();
        s.push_bytes(&bytes[..1]);
        assert_eq!(s.next_item(), InputItem::WouldBlock, "half a char yields nothing yet");
        s.push_bytes(&bytes[1..]);
        assert_eq!(s.next_item(), InputItem::Char('中'));
    }

    // ---------------- 编辑语义 ----------------

    /// 插入与提交：缓冲区累积，`Submit` 交出内容并清空。
    #[test]
    fn typing_then_submit() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        let actions = drive(&mut e, &mut h, &[
            InputItem::Char('h'), InputItem::Char('i'), InputItem::Submit,
        ]);
        assert!(matches!(actions[0], EditAction::Continue));
        assert!(matches!(actions[1], EditAction::Continue));
        assert_eq!(actions[2], EditAction::Submitted(b"hi".to_vec()));
        assert!(e.buffer.is_empty(), "buffer cleared after submit");
        assert_eq!(e.cursor, 0);
    }

    /// 光标在中间时插入必须**插在光标处**而非追加到末尾。
    #[test]
    fn insert_at_cursor_not_append() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        drive(&mut e, &mut h, &[
            InputItem::Char('a'), InputItem::Char('c'), InputItem::Left,
        ]);
        drive(&mut e, &mut h, &[InputItem::Char('b')]);
        assert_eq!(e.buffer, b"abc".to_vec());
        assert_eq!(e.cursor, 2, "cursor sits after the inserted char");
    }

    /// 退格删光标**前**一个；Delete 删光标**处**；
    /// 两者在边界（行首退格 / 行尾 Delete）不得 panic。
    #[test]
    fn backspace_and_delete_semantics() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        drive(&mut e, &mut h, &[InputItem::Char('a'), InputItem::Char('b')]);
        drive(&mut e, &mut h, &[InputItem::Backspace]);
        assert_eq!(e.buffer, b"a".to_vec());
        assert_eq!(e.cursor, 1);
        // 行尾 Delete：无事发生。
        drive(&mut e, &mut h, &[InputItem::Delete]);
        assert_eq!(e.buffer, b"a".to_vec());
        // 行首退格：**先把光标移到行首**，再退格应无事发生（不越界）。
        drive(&mut e, &mut h, &[InputItem::Home, InputItem::Backspace]);
        assert_eq!(e.buffer, b"a".to_vec(), "backspace at line start is a no-op");
        assert_eq!(e.cursor, 0);
        // 光标已在行首：Delete 删掉光标处那个字符。
        drive(&mut e, &mut h, &[InputItem::Delete]);
        assert_eq!(e.buffer, Vec::<u8>::new());
        // 空缓冲区上再退格/Delete 仍不得 panic。
        drive(&mut e, &mut h, &[InputItem::Backspace, InputItem::Delete]);
        assert_eq!(e.buffer, Vec::<u8>::new());
    }

    /// 光标移动必须夹在 `[0, len]`，超出边界的按键静默无效。
    #[test]
    fn cursor_movement_is_clamped() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        drive(&mut e, &mut h, &[InputItem::Char('a')]);
        drive(&mut e, &mut h, &[InputItem::Right, InputItem::Right, InputItem::Right]);
        assert_eq!(e.cursor, 1, "cannot move past the end");
        drive(&mut e, &mut h, &[InputItem::Left, InputItem::Left, InputItem::Left]);
        assert_eq!(e.cursor, 0, "cannot move before the start");
        drive(&mut e, &mut h, &[InputItem::End]);
        assert_eq!(e.cursor, 1);
        drive(&mut e, &mut h, &[InputItem::Home]);
        assert_eq!(e.cursor, 0);
    }

    // ---------------- 历史 ----------------

    /// 历史上翻：最新在最后，上翻从最新开始；到最旧即停（不绕回）。
    #[test]
    fn history_prev_walks_backwards_and_stops() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        e.push_history(b"one");
        e.push_history(b"two");
        drive(&mut e, &mut h, &[InputItem::HistoryPrev]);
        assert_eq!(e.buffer, b"two".to_vec(), "first up = most recent");
        drive(&mut e, &mut h, &[InputItem::HistoryPrev]);
        assert_eq!(e.buffer, b"one".to_vec());
        drive(&mut e, &mut h, &[InputItem::HistoryPrev]);
        assert_eq!(e.buffer, b"one".to_vec(), "at oldest, must not wrap");
    }

    /// 历史下翻：退出历史时必须恢复**进入历史前的草稿**，
    /// 而不是空行——否则用户打了一半的命令会静默丢失。
    #[test]
    fn history_next_restores_draft() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        e.push_history(b"old");
        drive(&mut e, &mut h, &[
            InputItem::Char('d'), InputItem::Char('r'), InputItem::Char('a'), InputItem::Char('f'), InputItem::Char('t'),
        ]);
        drive(&mut e, &mut h, &[InputItem::HistoryPrev]);
        assert_eq!(e.buffer, b"old".to_vec());
        drive(&mut e, &mut h, &[InputItem::HistoryNext]);
        assert_eq!(e.buffer, b"draft".to_vec(), "draft must come back");
    }

    /// 历史去重：与上一条相同不重复记；空行不记。
    #[test]
    fn history_dedupes_adjacent_and_skips_empty() {
        let mut e = Editor::new();
        e.push_history(b"x");
        e.push_history(b"x");
        assert_eq!(e.history.len(), 1, "adjacent duplicate not recorded");
        e.push_history(b"");
        assert_eq!(e.history.len(), 1, "empty line not recorded");
        e.push_history(b"y");
        assert_eq!(e.history.len(), 2);
    }

    /// 提交会把行记入历史（且清空历史游犯状态）。
    #[test]
    fn submit_records_history_and_resets_nav() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        drive(&mut e, &mut h, &[InputItem::Char('l'), InputItem::Char('s'), InputItem::Submit]);
        assert_eq!(e.history, vec![b"ls".to_vec()]);
        assert_eq!(e.nav.offset, None, "nav reset after submit");
    }

    // ---------------- 中断（^C） ----------------

    /// `^C` 必须中断当前行且**不提交**，缓冲区与历史都不得污染。
    #[test]
    fn interrupt_discards_line_without_submitting() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        drive(&mut e, &mut h, &[InputItem::Char('a'), InputItem::Char('b')]);
        let a = e.apply(InputItem::Interrupt, &mut h);
        assert_eq!(a, EditAction::Interrupted);
        assert!(e.buffer.is_empty());
        assert!(e.interrupted, "interrupt flag set for the caller");
        assert!(e.history.is_empty(), "interrupted line must NOT enter history");
    }

    // ---------------- 回显抑制（ADR-046 决策 3） ----------------

    /// 抑制回显时，**不得**调 host.redraw——否则口令会被画到屏幕上。
    #[test]
    fn echo_suppression_never_redraws() {
        let mut e = Editor::new().with_echo_suppressed(true);
        let mut h = FakeHost::default();
        let mut src = ByteSource::new();
        src.push_bytes(b"secret\r");
        let act = read_line(&mut e, &mut src, &mut h, |_| {});
        assert!(matches!(act, EditAction::Submitted(_)));
        assert!(h.redraws.is_empty(), "suppressed echo must not redraw");
    }

    /// 对照：未抑制时，编辑确实会重绘（证明上一测测的是抑制而非“根本不重绘”）。
    #[test]
    fn without_suppression_redraw_happens() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        let mut src = ByteSource::new();
        src.push_bytes(b"ab\r");
        let act = read_line(&mut e, &mut src, &mut h, |_| {});
        assert!(matches!(act, EditAction::Submitted(_)));
        assert!(!h.redraws.is_empty(), "normal editing redraws");
    }

    // ---------------- Tab 补全 ----------------

    /// 唯一候选时直接补完；命令位追一个空格。
    #[test]
    fn completion_single_candidate_completes_with_space() {
        let mut e = Editor::new();
        let mut h = FakeHost { completions: vec![b"echo".to_vec()], ..Default::default() };
        drive(&mut e, &mut h, &[InputItem::Char('e'), InputItem::Char('c')]);
        e.complete(&mut h);
        assert_eq!(e.buffer, b"echo ".to_vec());
    }

    /// 多候选时只补到最长公共前缀（不猜）。
    #[test]
    fn completion_multiple_candidates_completes_to_lcp() {
        let mut e = Editor::new();
        let mut h = FakeHost {
            completions: vec![b"mount".to_vec(), b"mouse".to_vec()],
            ..Default::default()
        };
        drive(&mut e, &mut h, &[InputItem::Char('m')]);
        e.complete(&mut h);
        assert_eq!(e.buffer, b"mou".to_vec(), "only the common prefix");
    }

    /// 环境变量位（`$` 前缀）补全必须保留 `$`，且**不**追空格。
    #[test]
    fn completion_preserves_dollar_and_no_trailing_space() {
        let mut e = Editor::new();
        let mut h = FakeHost { completions: vec![b"PATH".to_vec()], ..Default::default() };
        drive(&mut e, &mut h, &[InputItem::Char('e'), InputItem::Char('c'), InputItem::Char('h'), InputItem::Char('o'), InputItem::Char(' '), InputItem::Char('$')]);
        e.complete(&mut h);
        assert_eq!(e.buffer, b"echo $PATH".to_vec());
    }

    /// 无候选时缓冲区**不变**（不得静默清空用户输入）。
    #[test]
    fn completion_with_no_matches_leaves_buffer_alone() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        drive(&mut e, &mut h, &[InputItem::Char('z'), InputItem::Char('z')]);
        e.complete(&mut h);
        assert_eq!(e.buffer, b"zz".to_vec());
    }

    /// Smart-Case：全小写前缀忽略大小写；含大写则严格。
    #[test]
    fn smart_case_matching() {
        assert!(smart_case_match(b"ec", b"echo"));
        // 规则：**前缀含任一大写字母即严格匹配**。
        // 故「全小写」才宽松，「EC」虽全大写但**属于严格档**。
        assert!(!smart_case_match(b"EC", b"echo"), "uppercase prefix is strict: 'EC' != 'ec'");
        assert!(smart_case_match(b"EC", b"ECHO"), "strict match succeeds when equal");
        assert!(!smart_case_match(b"Ec", b"ECHO"), "mixed case is strict");
        assert!(smart_case_match(b"ECHO", b"ECHO"));
        assert!(!smart_case_match(b"xyz", b"echo"));
        assert!(!smart_case_match(b"echoo", b"echo"), "prefix longer than candidate");
    }

    // ---------------- 中间函数直接对拍 ----------------

    #[test]
    fn word_start_finds_last_token_boundary() {
        assert_eq!(word_start(b""), 0);
        assert_eq!(word_start(b"echo"), 0);
        assert_eq!(word_start(b"echo "), 5);
        assert_eq!(word_start(b"echo ab"), 5);
        assert_eq!(word_start(b"echo\tab"), 5);
    }

    #[test]
    fn longest_common_prefix_basics() {
        assert_eq!(longest_common_prefix(&[]), 0);
        assert_eq!(longest_common_prefix(&[b"abc".to_vec()]), 3);
        assert_eq!(longest_common_prefix(&[b"abc".to_vec(), b"abd".to_vec()]), 2);
        assert_eq!(longest_common_prefix(&[b"ab".to_vec(), b"abc".to_vec()]), 2);
        assert_eq!(longest_common_prefix(&[b"a".to_vec(), b"b".to_vec()]), 0);
    }

    // ---------------- 输入源的「补货」契约（L-2 真实缺陷回归） ----------------

    /// 一个**初始为空、只能靠 refill 供货**的输入源。
    ///
    /// 现实中 HID 键盘就是这样：`read_line` 进入时队列是空的，字符随按键
    /// 陆续到达。初版 `read_line` 在 `WouldBlock` 时只 yield 不调 refill，
    /// 于是这类源永远读不到东西——**这正是 L-2 首轮把 shell 弄死的缺陷**。
    struct DripSource {
        chunks: Vec<Vec<u8>>,
        buf: ByteSource,
        refills: usize,
    }

    impl InputSource for DripSource {
        fn next_item(&mut self) -> InputItem {
            self.buf.next_item()
        }
        fn refill(&mut self) -> bool {
            self.refills += 1;
            if self.chunks.is_empty() {
                return false;
            }
            let c = self.chunks.remove(0);
            self.buf.push_bytes(&c);
            true
        }
    }

    /// **回归**：输入不是一开始就齐的，而是逐次 refill 才到齐；
    /// `read_line` 必须在空队列时主动 refill，否则永远拿不到这一行。
    #[test]
    fn read_line_refills_empty_source_until_line_arrives() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        let mut src = DripSource {
            chunks: vec![b"o".to_vec(), b"k".to_vec(), b"\r".to_vec(), vec![0x04]],
            buf: ByteSource::new(),
            refills: 0,
        };
        let act = read_line(&mut e, &mut src, &mut h, |_| {});
        assert_eq!(act, EditAction::Submitted(b"ok".to_vec()));
        assert!(src.refills >= 3, "must have refilled for each chunk");
    }

    /// `ByteSource::refill` 如实报「无新增」，不伪造数据。
    #[test]
    fn byte_source_refill_reports_no_new_bytes() {
        let mut s = ByteSource::new();
        assert!(!s.refill());
        s.push_bytes(b"x");
        assert!(!s.refill(), "push_bytes is the caller's job, not refill's");
    }

    /// 输入源**彻底枯竭**时，`read_line` 必须诚实返回（不得死循环）。
    /// 用 Eof 终结，证明循环有出口。
    #[test]
    fn read_line_terminates_on_eof_from_empty_source() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        let mut src = DripSource {
            chunks: vec![b"a".to_vec(), vec![0x04]], // 'a' then ^D
            buf: ByteSource::new(),
            refills: 0,
        };
        let act = read_line(&mut e, &mut src, &mut h, |_| {});
        assert_eq!(act, EditAction::Eof(b"a".to_vec()));
    }

    // ---------------- L-4：^C 投递 SIGINT（ADR-046 §1.2） ----------------

    /// **默认不投递**：`interrupt_target` 为 `None` 时 `^C` 只作废当前行。
    ///
    /// 这是 `login` 与「shell 当前无前台子进程」两种情形的共同语义——
    /// 没有目标可打，不该猜一个。
    #[test]
    fn interrupt_without_target_only_discards_line() {
        let mut e = Editor::new();
        assert_eq!(e.interrupt_target, None, "target must default to None");
        let mut h = FakeHost::default();
        drive(&mut e, &mut h, &[InputItem::Char('x')]);
        assert_eq!(e.apply(InputItem::Interrupt, &mut h), EditAction::Interrupted);
        assert!(e.buffer.is_empty());
    }

    /// **目标由调用方设置**：`with_interrupt_target` 必须如实保存它。
    ///
    /// 本测试锁定「库不猜目标」这一契约的形状：目标是一个**显式输入**，
    /// 不是一个由库自行推导的值。
    #[test]
    fn interrupt_target_is_whatever_the_caller_set() {
        let e = Editor::new().with_interrupt_target(Some(4242));
        assert_eq!(e.interrupt_target, Some(4242));
        let e2 = Editor::new().with_interrupt_target(None);
        assert_eq!(e2.interrupt_target, None);
    }

    /// **投递失败不改变结论**：目标 pid 不存在（宿主上任何 pid 都不存在）时，
    /// `^C` 仍必须如实返回 `Interrupted` 并清空缓冲区。
    ///
    /// 这模拟真实竞态：子进程恰在用户按键前退出，`kill` 返回 NotFound。
    /// 用户要的是「回到干净提示符」，已达成就够了——不该因此报错或卡住。
    #[test]
    fn interrupt_with_dead_target_still_interrupts() {
        let mut e = Editor::new().with_interrupt_target(Some(999_999));
        let mut h = FakeHost::default();
        drive(&mut e, &mut h, &[InputItem::Char('a'), InputItem::Char('b')]);
        assert_eq!(e.apply(InputItem::Interrupt, &mut h), EditAction::Interrupted);
        assert!(e.buffer.is_empty(), "buffer cleared even if kill failed");
        assert!(e.interrupted);
    }

    /// 端到端（库内）：`ByteSource` 里的 `0x03` 经 `apply` 必须走到投递分支。
    ///
    /// 与 `interrupt_with_dead_target_still_interrupts` 的区别：那条直接调
    /// `apply`，这条从**真实字节**出发，证明 `0x03` 的解码与投递是同一条链。
    #[test]
    fn ctrl_c_byte_reaches_the_interrupt_path() {
        let mut e = Editor::new().with_interrupt_target(Some(999_999));
        let mut h = FakeHost::default();
        let mut src = ByteSource::new();
        src.push_bytes(&[b'a', 0x03]);
        assert_eq!(src.next_item(), InputItem::Char('a'));
        assert_eq!(src.next_item(), InputItem::Interrupt);
        let act = e.apply(InputItem::Interrupt, &mut h);
        assert_eq!(act, EditAction::Interrupted);
        assert!(e.buffer.is_empty(), "0x03 must discard the pending line");
    }

    /// **`^D`（EOF）不得被误当中断**：两者相邻（0x03/0x04），必须分清。
    #[test]
    fn ctrl_d_is_eof_not_interrupt() {
        let mut e = Editor::new().with_interrupt_target(Some(999_999));
        let mut h = FakeHost::default();
        let mut src = ByteSource::new();
        src.push_bytes(&[b'x', 0x04]);
        assert_eq!(src.next_item(), InputItem::Char('x'));
        assert_eq!(src.next_item(), InputItem::Eof);
        // 先把 'x' 真的送进编辑器，再送 EOF——否则测的是空缓冲区。
        assert_eq!(e.apply(InputItem::Char('x'), &mut h), EditAction::Continue);
        assert_eq!(e.apply(InputItem::Eof, &mut h), EditAction::Eof(b"x".to_vec()));
        assert!(!e.interrupted, "EOF must not set the interrupt flag");
    }
    // ---------------- L-3：login 的裁剪接口（ADR-046 §4 条 3） ----------------

    /// 记录 `completions` 是否被调用过的 host。
    ///
    /// 这是 L-3 裁剪约束的**可执行证据**：若 `read_line_plain` 真的不给补全，
    /// 则宿主永远不该收到补全请求。
    #[derive(Default)]
    struct CountingHost {
        completion_calls: usize,
        redraws: usize,
        completions: Vec<Vec<u8>>,
    }

    impl EditorHost for CountingHost {
        fn write(&mut self, _bytes: &[u8]) {}
        fn redraw(&mut self, _buffer: &[u8], _cursor: usize) {
            self.redraws += 1;
        }
        fn completions(&mut self, _word: &[u8], _is_command: bool) -> Vec<Vec<u8>> {
            self.completion_calls += 1;
            self.completions.clone()
        }
    }

    /// **裁剪约束（核心）**：`read_line_plain` 期间按 Tab，宿主**不得**收到补全请求。
    ///
    /// 若本测试失败，说明 login 拿到了它不该有的补全能力——正是 §4 条 3 要防的。
    #[test]
    fn plain_read_never_requests_completions() {
        let mut e = Editor::new();
        let mut h = CountingHost {
            completions: vec![b"shouldneverbeused".to_vec()],
            ..Default::default()
        };
        let mut src = ByteSource::new();
        // 'a' Tab 'b' Enter —— 中间夹一个 Tab
        src.push_bytes(&[b'a', 0x09, b'b', b'\n']);
        let act = read_line_plain(
            &mut e,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: false },
        );
        assert_eq!(act, EditAction::Submitted(b"ab".to_vec()));
        assert_eq!(h.completion_calls, 0, "plain read must not ask for completions");
    }

    /// **裁剪约束**：`read_line_plain` 期间按 ↑，**不得**召回任何历史。
    ///
    /// 验证方式：预先塞满历史，再按 ↑。若实现漏给了历史能力，
    /// 缓冲区就会变成历史里那条，而不是空。
    #[test]
    fn plain_read_never_recalls_history() {
        let mut e = Editor::new();
        e.push_history(b"secret-from-history");
        let mut h = CountingHost::default();
        let mut src = ByteSource::new();
        // ↑ 然后输入 'x' 再回车：若历史生效，结果会是 "secret-from-historyx"
        src.push_bytes(b"\x1b[Ax\n");
        let act = read_line_plain(
            &mut e,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: false },
        );
        assert_eq!(
            act,
            EditAction::Submitted(b"x".to_vec()),
            "history must not be reachable through the plain interface"
        );
    }

    /// `echo: false` 时**任何**输入都不上屏（口令不回显）。
    ///
    /// 判据：`redraw` 一次都不许被调——因为「把口令画到屏幕上」正是要防的。
    #[test]
    fn plain_read_with_echo_off_never_redraws() {
        let mut e = Editor::new();
        let mut h = CountingHost::default();
        let mut src = ByteSource::new();
        src.push_bytes(b"hunter2\n");
        let act = read_line_plain(
            &mut e,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: true },
        );
        assert_eq!(act, EditAction::Submitted(b"hunter2".to_vec()));
        assert_eq!(h.redraws, 0, "password must never reach the screen");
    }

    /// 对照：`echo: true` 时确实会重绘（证明上一测测的是抑制，而非「从不重绘」）。
    #[test]
    fn plain_read_with_echo_on_does_redraw() {
        let mut e = Editor::new();
        let mut h = CountingHost::default();
        let mut src = ByteSource::new();
        src.push_bytes(b"alice\n");
        let act = read_line_plain(
            &mut e,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: false },
        );
        assert_eq!(act, EditAction::Submitted(b"alice".to_vec()));
        assert!(h.redraws > 0, "username should be visible while typing");
    }

    /// 退格在朴素接口下照常工作（login 依赖它修正打错的用户名）。
    #[test]
    fn plain_read_supports_backspace() {
        let mut e = Editor::new();
        let mut h = CountingHost::default();
        let mut src = ByteSource::new();
        src.push_bytes(&[b'a', b'b', 0x7f, b'c', b'\n']);
        let act = read_line_plain(
            &mut e,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: false },
        );
        assert_eq!(act, EditAction::Submitted(b"ac".to_vec()));
    }

    /// **L-3 回归**：连续两次「朴素读行」（用户名 + 口令）都必须返回。
    ///
    /// 真实内核上曾出现：第一次读（用户名）正常返回，第二次读（口令）
    /// **永久挂住**。单测原本没覆盖它——因为既有测试总是新建一个 source，
    /// 而 login 是**同一进程里连续两次**读。本测试用同一个 source 连读两次。
    #[test]
    fn two_consecutive_plain_reads_both_return() {
        let mut src = ByteSource::new();
        src.push_bytes(b"alice\r");
        src.push_bytes(b"alicepw\r");
        let mut h = CountingHost::default();

        let mut e1 = Editor::new();
        let a1 = read_line_plain(
            &mut e1,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: false },
        );
        assert_eq!(a1, EditAction::Submitted(b"alice".to_vec()));

        let mut e2 = Editor::new();
        let a2 = read_line_plain(
            &mut e2,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: true },
        );
        assert_eq!(a2, EditAction::Submitted(b"alicepw".to_vec()));
    }

    /// 同一 `Editor` 实例连读两次也必须都能返回。
    #[test]
    fn same_editor_two_plain_reads_both_return() {
        let mut src = ByteSource::new();
        src.push_bytes(b"alice\r");
        src.push_bytes(b"alicepw\r");
        let mut h = CountingHost::default();
        let mut e = Editor::new();
        let a1 = read_line_plain(
            &mut e,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: false },
        );
        assert_eq!(a1, EditAction::Submitted(b"alice".to_vec()));
        let a2 = read_line_plain(
            &mut e,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: true },
        );
        assert_eq!(a2, EditAction::Submitted(b"alicepw".to_vec()));
    }
    /// **L-3 真相测试**：模拟 login 的真实时序——口令字符与 CR **分两批**到达，
    /// 中间输入源报告「已补到货」。这精确复现真实内核上的挂住。
    #[test]
    fn plain_read_echo_off_with_refill_between_chars_and_cr() {
        struct TwoPhase {
            inner: ByteSource,
            phase: usize,
        }
        impl InputSource for TwoPhase {
            fn next_item(&mut self) -> InputItem {
                self.inner.next_item()
            }
            fn refill(&mut self) -> bool {
                // 第一批：口令字符。第二批：CR。之后再也没有。
                match self.phase {
                    0 => {
                        self.inner.push_bytes(b"alicepw");
                        self.phase = 1;
                        true
                    }
                    1 => {
                        self.inner.push_bytes(b"\r");
                        self.phase = 2;
                        true
                    }
                    _ => false,
                }
            }
        }
        let mut e = Editor::new();
        let mut h = CountingHost::default();
        let mut src = TwoPhase { inner: ByteSource::new(), phase: 0 };
        let act = read_line_plain(
            &mut e,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: true },
        );
        assert_eq!(act, EditAction::Submitted(b"alicepw".to_vec()));
    }

    /// 同上，但用**用户名**语义（不抑制回显）——对照，验证不是回显抑制独有的问题。
    #[test]
    fn plain_read_echo_on_with_refill_between_chars_and_cr() {
        struct TwoPhase {
            inner: ByteSource,
            phase: usize,
        }
        impl InputSource for TwoPhase {
            fn next_item(&mut self) -> InputItem {
                self.inner.next_item()
            }
            fn refill(&mut self) -> bool {
                match self.phase {
                    0 => {
                        self.inner.push_bytes(b"alice");
                        self.phase = 1;
                        true
                    }
                    1 => {
                        self.inner.push_bytes(b"\r");
                        self.phase = 2;
                        true
                    }
                    _ => false,
                }
            }
        }
        let mut e = Editor::new();
        let mut h = CountingHost::default();
        let mut src = TwoPhase { inner: ByteSource::new(), phase: 0 };
        let act = read_line_plain(
            &mut e,
            &mut src,
            &mut h,
            |_| {},
            PlainLineOptions { suppress_echo: false },
        );
        assert_eq!(act, EditAction::Submitted(b"alice".to_vec()));
    }
    /// UTF-8 插入：一个中文字占 3 字节，光标必须按**字节**推进。
    #[test]
    fn utf8_char_insert_advances_cursor_by_byte_len() {
        let mut e = Editor::new();
        let mut h = FakeHost::default();
        drive(&mut e, &mut h, &[InputItem::Char('中')]);
        assert_eq!(e.buffer, "中".as_bytes().to_vec());
        assert_eq!(e.cursor, 3, "cursor is a byte offset");
    }
}