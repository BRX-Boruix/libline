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
            InputItem::Complete => {
                self.complete(host);
                EditAction::Continue
            }
            InputItem::HistoryPrev => {
                self.history_prev();
                EditAction::Continue
            }
            InputItem::HistoryNext => {
                self.history_next();
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
            // 无键可读时让出 CPU（避免空转）。
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