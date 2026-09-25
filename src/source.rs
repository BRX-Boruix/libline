//! \u8f93\u5165\u6e90\u62bd\u8c61\uff1a\u672c\u5e93\u5bf9\u5916**\u552f\u4e00**\u7684\u8f93\u5165\u4f9d\u8d56\u3002
//!
//! # \u4e3a\u4f55\u8981\u62bd\u8c61\uff08ADR-046 \u00a74 \u6761 5\uff09
//!
//! ADR-045 \u4e8b\u4ef6\u6d41\u843d\u5730\u540e\uff0c\u884c\u7f16\u8f91\u7684\u8f93\u5165\u4f1a\u4ece\u300c\u5b57\u8282\u6d41\u300d\u53d8\u4e3a\u300c\u4e8b\u4ef6\u6d41\u300d\u3002
//! \u4e24\u6761\u8def\u5f84\u82e5\u76f4\u63a5\u5199\u6b7b\u5728\u7f16\u8f91\u5668\u91cc\uff0c\u5c31\u4f1a\u5f62\u6210\u4e00\u4e2a\u5fc5\u987b\u540c\u65f6\u6539\u7684\u8026\u5408\u70b9\u3002
//! \u6545\u672c\u5e93\u53ea\u4f9d\u8d56\u4e00\u4e2a\u300c\u53d6\u4e0b\u4e00\u4e2a\u8f93\u5165\u5355\u5143\u300d\u7684\u7a84\u63a5\u53e3\uff1a
//! **\u80cc\u540e\u662f\u5b57\u8282\u6d41\u8fd8\u662f\u4e8b\u4ef6\u6d41\u7531\u8c03\u7528\u65b9\u51b3\u5b9a\uff0c\u7f16\u8f91\u5668\u672c\u4f53\u4e0d\u77e5\u60c5\u3002**

/// \u4e00\u4e2a\u8f93\u5165\u5355\u5143\uff08\u8f93\u5165\u6e90\u5bf9\u7f16\u8f91\u5668\u7684\u627f\u8bfa\uff09\u3002
///
/// **\u4e3a\u4f55\u4e0d\u76f4\u63a5\u7528 `u8`**\uff1a\u5b57\u8282\u6d41\u80fd\u8868\u8fbe\u7684\u662f\u4e00\u4e2a\u5b57\u8282\uff0c\u4e8b\u4ef6\u6d41\u80fd\u8868\u8fbe\u7684\u662f
/// \u300c\u6309\u4e0b\u4e86\u54ea\u4e2a\u952e\u300d\u3002\u4e24\u8005\u7684\u5171\u540c\u5206\u6bcd\u662f\u300c\u4e00\u4e2a\u53ef\u89e3\u91ca\u7684\u8f93\u5165\u52a8\u4f5c\u300d\uff0c\u6545\u62bd\u8c61\u5230\u8fd9\u4e00\u5c42\u3002
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputItem {
    /// \u4e00\u4e2a\u53ef\u6253\u5370\u5b57\u7b26\uff08\u5df2\u89e3\u7801\uff0c\u542b\u591a\u5b57\u8282 UTF-8 \u7684\u5355\u4e2a\u5b57\u7b26\uff09\u3002
    Char(char),
    /// \u56de\u8f66\uff08\u63d0\u4ea4\u5f53\u524d\u884c\uff09\u3002
    Submit,
    /// \u9000\u683c\uff08\u5220\u9664\u5149\u6807\u524d\u4e00\u4e2a\u5b57\u7b26\uff09\u3002
    Backspace,
    /// \u5220\u9664\uff08\u5220\u9664\u5149\u6807\u5904\u5b57\u7b26\uff09\u3002
    Delete,
    /// \u5149\u6807\u5de6\u79fb\u3002
    Left,
    /// \u5149\u6807\u53f3\u79fb\u3002
    Right,
    /// \u5149\u6807\u79fb\u5230\u884c\u9996\u3002
    Home,
    /// \u5149\u6807\u79fb\u5230\u884c\u5c3e\u3002
    End,
    /// \u8865\u5168\uff08Tab\uff09\u3002
    Complete,
    /// \u5386\u53f2\u4e0a\u4e00\u6761\uff08\u2191\uff09\u3002
    HistoryPrev,
    /// \u5386\u53f2\u4e0b\u4e00\u6761\uff08\u2193\uff09\u3002
    HistoryNext,
    /// \u6253\u65ad\uff08`^C`\uff0c0x03\uff09\uff1a\u8bbe\u7f6e\u4e2d\u65ad\u6807\u5fd7\uff0c\u4e0d\u63d0\u4ea4\u5f53\u524d\u884c\u3002
    Interrupt,
    /// \u6587\u4ef6\u7ed3\u675f\uff08\u8f93\u5165\u6e90\u5df2\u8017\u5c3d\uff09\u3002
    Eof,
    /// \u672c\u6b21\u672a\u53d6\u5230\u5355\u5143\uff08\u4f8b\u5982\u952e\u76d8\u7a7a\u95f2\uff09\uff0c\u8c03\u7528\u65b9\u53ef\u91cd\u8bd5\u3002
    WouldBlock,
}

/// \u8f93\u5165\u6e90\uff1a\u7f16\u8f91\u5668\u6bcf\u6b21\u9700\u8981\u4e0b\u4e00\u4e2a\u8f93\u5165\u5355\u5143\u65f6\u8c03\u7528\u5b83\u3002
///
/// **object-safe**\uff1a\u5b9e\u73b0\u8005\u53ef\u6301\u72b6\u6001\uff08\u5982\u9000\u5316\u4e2d\u7684\u591a\u5b57\u8282 UTF-8\uff09\uff0c
/// \u6545\u7528 `&mut self` \u800c\u975e `&self`\u3002
pub trait InputSource {
    /// \u53d6\u4e0b\u4e00\u4e2a\u8f93\u5165\u5355\u5143\u3002\u65e0\u8f93\u5165\u53ef\u53d6\u65f6\u8fd4\u56de [`InputItem::WouldBlock`]\uff1b
    /// \u8f93\u5165\u5df2\u7ec8\u7ed3\u65f6\u8fd4\u56de [`InputItem::Eof`]\u3002
    fn next_item(&mut self) -> InputItem;

    /// 补充输入。返回 `true` 表示**新增**了可用字节；`false` 表示**本次没有得到任何东西**。
    ///
    /// **为何这个钩子必不可少**：输入源的字节来自外部（键盘、事件总线），
    /// 而 `libline` 不应知道怎么取。若 `next_item` 返回 `WouldBlock` 后
    /// 只能空转等待，输入永远不会再来——**这正是 L-2 首轮的真实缺陷**：
    /// shell 只在进 `read_line` 前泵一次 stdin，队列排空后再没人调 `read`，
    /// 于是用户后续按键全部丢失（实测：登录后连提示符都不再出现）。
    /// 补充的责任在**输入源自己**，故在 trait 上开一个钩子。
    ///
    /// # 契约（实现者必须遵守；违反会**静默丢键**）
    ///
    /// **`refill` 不得用 `false` 表达「暂时没有数据」。**
    ///
    /// 对一个**活跃的**输入源（背后是键盘 / 事件总线等随时可能来数据的设备），
    /// `refill` 必须**阻塞到有数据、或到真正的终止条件**（真错误 / 真 EOF）为止，
    /// 期间可以自旋重试。只有**确实无法再产生数据**时才返回 `false`。
    ///
    /// ## 为什么（本内核的具体机制，不是风格问题）
    ///
    /// `read_line`（`editor.rs`）在 `next_item` 返回 [`InputItem::WouldBlock`] 时：
    ///
    /// ```text
    /// if src.refill() { continue; }        // 拿到东西就立刻重试
    /// let _ = libsys::yield_now();         // 否则让出 CPU 再重试
    /// continue;
    /// ```
    ///
    /// 若 `refill` 在「暂时没数据」时返回 `false`，执行流就落到 `yield_now()` 上。
    /// 而本内核的键盘唤醒是**单等待者**语义：只有**正阻塞在内核 `read` 里**的进程
    /// 才登记为 `KBD_WAITER`，键盘中断也只唤醒**登记过的那个** pid
    /// （`kernel/crates/task/src/scheduler.rs`）。`yield_now()` 是**纯让出、不登记等待者**，
    /// 于是出现**无人登记**的窗口：击键躺在键盘队列里，`wake_kbd` 无处可唤醒 ⇒
    /// **按键永久丢失**（症状：口令行回车不到达、登录后提示符不再出现）。详见
    /// `docs/TODO/terminal-input.md` §6.7。
    ///
    /// 故「原地重试直到有数据」**不是**防御性写法，而是本内核下的**正确性要求**。
    ///
    /// ## 参考实现
    ///
    /// * `shell/src/main.rs`、`login/src/main.rs` 的 `refill`：`WouldBlock` 上 `continue`，
    ///   仅真错误 / 真 EOF 返回 `false`——**符合契约**；
    /// * [`ByteSource::refill`]：字节由调用方 `push_bytes` 推入，自身**无源可补**，
    ///   故恒返回 `false`——这是**合法**的（它确实无法再产生数据），**不是违约**。
    ///   但此时 `read_line` 会 `yield_now()` 自旋，故 `ByteSource` **只适用于**
    ///   调用方自己会**再次推入**字节的场景（如宿主测试、逐块喂入），
    ///   **不适用于**「背后是活设备」的输入源。
    ///
    /// ## 未来实现者（尤其事件流，ADR-045 阶段 2）
    ///
    /// 事件流消费端**必须**遵守本契约：在事件未到达时阻塞（或自旋）在内核等待原语上，
    /// 而**不是**返回 `false` 让 `read_line` 去 `yield_now()`——否则会原样复现 §6.7 的缺陷。
    fn refill(&mut self) -> bool;
}

/// \u5b57\u8282\u6d41\u8f93\u5165\u6e90\uff1a\u628a\u5b57\u8282\u6d41\u89e3\u7801\u4e3a\u8f93\u5165\u5355\u5143\uff08**\u5f53\u524d\u5b9e\u4f53**\uff09\u3002
///
/// \u8d1f\u8d23\u628a\u539f\u59cb\u5b57\u8282\u89e3\u6790\u6210\u8f93\u5165\u5355\u5143\uff1aANSI CSI \u5e8f\u5217\u2192\u5149\u6807/\u5386\u53f2
/// \u52a8\u4f5c\uff0c\u63a7\u5236\u5b57\u8282\u2192\u9000\u683c/\u63d0\u4ea4/\u4e2d\u65ad\u3002
///
/// **\u4e3a\u4f55\u628a\u89e3\u6790\u653e\u5728\u8fd9\u91cc\u800c\u4e0d\u5728\u7f16\u8f91\u5668\u91cc**\uff1a\u8fd9\u662f**\u5b57\u8282\u6d41\u7279\u6709**\u7684\u5951\u7ea6
/// \uff08\u4e8b\u4ef6\u6d41\u8f93\u5165\u6e90\u65e0\u9700\u518d\u89e3\u6792\u5b57\u8282\uff09\u3002\u653e\u5728\u7f16\u8f91\u5668\u91cc\u4f1a\u8ba9\u672a\u6765\u7684\u4e8b\u4ef6\u6e90
/// \u88ab\u8feb\u7ee7\u627f\u4e00\u5957\u5b83\u4e0d\u9700\u8981\u7684\u5b57\u8282\u72b6\u6001\u673a\u3002
#[derive(Debug, Default)]
pub struct ByteSource {
    /// \u5f85\u5904\u7406\u7684\u539f\u59cb\u5b57\u8282\u7f13\u51b2\u3002
    pending: alloc::vec::Vec<u8>,
    /// ANSI \u89e3\u6790\u72b6\u6001\uff1a0=\u666e\u901a 1=\u5df2\u89c1 ESC 2=CSI \u53c2\u6570 3=ESC O \u5e8f\u5217\u3002
    esc_state: u8,
    /// CSI \u53c2\u6570\u7f13\u5b58\uff08\u4f9b `~` \u7ec8\u6b62\u7b26\u533a\u5206 1~/3~/4~\uff09\u3002
    csi_param: alloc::vec::Vec<u8>,
    /// UTF-8 \u9000\u5316\u4e2d\u7684\u5b57\u8282\u3002
    utf8_buf: alloc::vec::Vec<u8>,
}

impl ByteSource {
    /// \u7a7a\u8f93\u5165\u6e90\u3002
    pub fn new() -> Self {
        Self::default()
    }

    /// \u5411\u8f93\u5165\u6e90\u6ce8\u5165\u539f\u59cb\u5b57\u8282\uff08\u7531\u8c03\u7528\u65b9\u7684 `read` \u63d0\u4f9b\uff09\u3002
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
    }

    /// \u662f\u5426\u8fd8\u6709\u672a\u5904\u7406\u5b57\u8282\u3002
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
}

impl InputSource for ByteSource {
    /// `ByteSource` 的字节由调用方 `push_bytes` 推入，自身无源可补：
    /// 返回 false 表示「没有新增字节」，不伪造。
    fn refill(&mut self) -> bool {
        false
    }

    /// \u53d6\u4e0b\u4e00\u4e2a\u8f93\u5165\u5355\u5143\u3002
    ///
    /// **\u5173\u952e\uff1a\u5185\u90e8\u5faa\u73af\u76f4\u5230\u4ea7\u51fa\u4e00\u4e2a\u771f\u5355\u5143\uff0c\u6216\u961f\u5217\u771f\u7684\u7a7a\u4e86\u3002**
    ///
    /// \u82e5\u53ea\u6d88\u8017\u4e00\u4e2a\u5b57\u8282\u5c31\u8fd4\u56de\uff0c\u90a3\u4e48\u300c\u521a\u5403\u4e0b ESC\uff0c\u8fd8\u5728\u7b49\u540e\u7eed\u5b57\u8282\u300d\u4e0e
    /// \u300c\u961f\u5217\u672c\u6765\u5c31\u662f\u7a7a\u7684\u300d\u4f1a\u90fd\u8868\u73b0\u4e3a `WouldBlock`\uff0c\u8c03\u7528\u65b9\u65e0\u6cd5\u533a\u5206\u800c\u4e22\u5931\u771f\u5b9e\u52a8\u4f5c
    /// \uff08\u521d\u7248\u5c31\u662f\u8fd9\u4e2a\u9519\uff1a`\x1b[A` \u8f93\u5165\u540e\u4e0a\u7ffb\u52a8\u4f5c\u88ab\u541e\u6389\uff0c6 \u9879\u6d4b\u8bd5\u53d8\u7ea2\uff09\u3002
    ///
    /// \u800c\u5bf9**\u771f\u6b63\u7a7a\u961f\u5217**\u8fd4\u56de `WouldBlock` \u662f\u5fc5\u8981\u7684\uff1a\u5b83\u8ba9 `read_line` \u80fd\u8ba9\u51fa CPU
    /// \u800c\u4e0d\u7a7a\u8f6c\uff08\u952e\u76d8\u7a7a\u95f2\u65f6\u4e0d\u80fd\u70e7\u6ee1 CPU\uff09\u3002
    fn next_item(&mut self) -> InputItem {
        loop {
            // \u771f\u7a7a\uff1a\u65e0\u5b57\u8282\u53ef\u53d6\u2014\u2014\u8fd9\u624d\u662f\u771f\u6b63\u7684 WouldBlock\u3002
            if self.pending.is_empty() && self.utf8_buf.is_empty() {
                return InputItem::WouldBlock;
            }
            let Some(c) = self.pending.first().copied() else {
                // \u961f\u5217\u7a7a\u4f46\u6709\u672a\u5b8c\u6210\u7684 UTF-8 \u5e8f\u5217\uff1a\u7b49\u540e\u7eed\u5b57\u8282\u3002
                return InputItem::WouldBlock;
            };
            self.pending.remove(0);
            let item = decode_byte(self, c);
            if !matches!(item, InputItem::WouldBlock) {
                return item;
            }
            // \u8be5\u5b57\u8282\u5c5e\u4e8e\u5e8f\u5217\u7684\u4e2d\u95f4\u6001\uff1a\u7ee7\u7eed\u6d88\u8017\u540e\u7eed\u5b57\u8282\u3002
        }
    }
}

/// \u5355\u4e2a\u5b57\u8282 \u2192 \u8f93\u5165\u5355\u5143\uff08\u542b ANSI/UTF-8 \u72b6\u6001\u673a\uff09\u3002
///
/// \u62c6\u4e3a\u72ec\u7acb\u51fd\u6570\u4f7f\u5176**\u53ef\u5355\u72ec\u5bf9\u62cd**\uff1a\u8f93\u5165\u4e00\u4e32\u5b57\u8282\uff0c\u65ad\u8a00\u5f97\u5230\u7684\u5355\u5143\u5e8f\u5217\u3002
pub(crate) fn decode_byte(src: &mut ByteSource, c: u8) -> InputItem {
    // --- ANSI \u5e8f\u5217\u72b6\u6001\u673a ---
    if src.esc_state != 0 {
        match src.esc_state {
            1 => {
                if c == b'[' {
                    src.esc_state = 2;
                    src.csi_param.clear();
                } else if c == b'O' {
                    src.esc_state = 3;
                } else {
                    src.esc_state = 0;
                }
            }
            2 => {
                if c.is_ascii_digit() || c == b';' {
                    src.csi_param.push(c);
                } else if (0x40..=0x7E).contains(&c) {
                    src.esc_state = 0;
                    return csi_to_item(&src.csi_param, c);
                } else if !(0x20..=0x3F).contains(&c) {
                    src.esc_state = 0;
                }
            }
            3 => {
                src.esc_state = 0;
            }
            _ => {}
        }
        return InputItem::WouldBlock;
    }

    match c {
        0x1B => {
            src.esc_state = 1;
            InputItem::WouldBlock
        }
        b'\r' | b'\n' => InputItem::Submit,
        0x7F | 0x08 => InputItem::Backspace,
        0x09 => InputItem::Complete,
        0x03 => InputItem::Interrupt,
        0x04 => InputItem::Eof,
        _ if c < 0x20 => InputItem::WouldBlock,
        _ => finish_utf8(src, c),
    }
}

/// UTF-8 \u9000\u5316\uff1a\u6536\u9f50\u591a\u5b57\u8282\u5e8f\u5217\u540e\u8fd4\u56de\u5355\u4e2a `char`\u3002
pub(crate) fn finish_utf8(src: &mut ByteSource, c: u8) -> InputItem {
    src.utf8_buf.push(c);
    match core::str::from_utf8(&src.utf8_buf) {
        Ok(s) => {
            let ch = s.chars().next().unwrap_or('\u{fffd}');
            src.utf8_buf.clear();
            InputItem::Char(ch)
        }
        Err(_) => {
            // \u5c1a\u672a\u6536\u9f50\uff1a\u7ee7\u7eed\u7b49\u5f85\u540e\u7eed\u5b57\u8282\uff08\u6700\u591a 4 \u5b57\u8282\uff09\u3002
            if src.utf8_buf.len() >= 4 {
                src.utf8_buf.clear();
            }
            InputItem::WouldBlock
        }
    }
}

/// CSI \u53c2\u6570 + \u7ec8\u6b62\u7b26 \u2192 \u8f93\u5165\u5355\u5143\u3002
pub(crate) fn csi_to_item(param: &[u8], final_byte: u8) -> InputItem {
    match final_byte {
        b'A' => InputItem::HistoryPrev,
        b'B' => InputItem::HistoryNext,
        b'C' => InputItem::Right,
        b'D' => InputItem::Left,
        b'H' => InputItem::Home,
        b'F' => InputItem::End,
        b'~' => match param.first() {
            Some(b'1') => InputItem::Home,
            Some(b'4') => InputItem::End,
            Some(b'3') => InputItem::Delete,
            _ => InputItem::WouldBlock,
        },
        _ => InputItem::WouldBlock,
    }
}