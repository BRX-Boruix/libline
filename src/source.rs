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

/// 事件流输入源：字节来自 `/devices/input/events`（I-EVENTS 阶段 2，ADR-045）。
///
/// # 它与 [`ByteSource`] 的关系（S13 单一语义路径）
///
/// **不是**第二套解码器——内部的 `inner: ByteSource` 与字节流路径**完全同一个**，
/// 转义序列、退格、提交等语义一字不差。二者**唯一的差别**是：
///
/// | | 字节从哪来 |
/// | --- | --- |
/// | `ByteSource`（旧路径） | 调用方 `read` fd 0（内核「键盘→stdin」直连） |
/// | `EventSource`（本类型） | 调用方 `read` 事件节点，经 keymap 转换层产出字节 |
///
/// 故本类型只替换**取字节**这一件事，编辑语义零重复（S28）。
///
/// # 为何 keymap 在用户态（ADR-045 决策 3）
///
/// 内核只交**原始 scancode + 释放事件**，不认识字符。转换由
/// `libsys::event::decode_into` 完成——它是内核 `decode_key` 的逐分支镜像，
/// 且**释放事件**使「Shift 按住」可被用户态正确表达（字节流形态下被丢弃）。
///
/// # 阻塞语义（遵守 [`InputSource::refill`] 的契约）
///
/// [`read_into`](libsys::event::EventSourceReader::read_into) 内部走的是**阻塞**
/// `read`：内核在事件环为空时登记等待者并挂起本进程（`input_event_stream()`
/// 为真的节点），IRQ1 到达后唤醒。故 `false` 只会在**真错误**时出现——
/// 符合「不得用 `false` 表达『暂时没数据』」的契约，**不会**出现 §6.7 的丢键窗口。
/// 「取一批事件字节」的抽象（[`EventSource`] 对外的唯一依赖）。
///
/// # 为何要这一层（S23 纯函数优先 / S06 单一渲染点）
///
/// 真实实现走 `/devices/input/events` 的阻塞 `read`，**只在 QEMU 内可用**；
/// 宿主上 `syscall` 返回垃圾。若 [`EventSource`] 直接依赖具体类型，则
/// 「事件源与字节源是否等价」这条**最重要的判据**将无法在宿主测试——
/// 只能靠肉眼看串口，正是本项目反复吃亏的模式（§6.12.7/§6.12.8）。
///
/// 抽出本 trait 后，宿主测试可注入**可控的事件字节序列**，从而逐字节比对
/// 两条输入路径的产物。
pub trait EventBytes {
    /// 取一批原始**事件记录字节**（16 的整数倍），追加到 `out`。
    ///
    /// 返回 `Ok(())` 表示「本次调用已完成（可能有、也可能没有新字节）」；
    /// `Err` 表示取字节本身失败（如节点不存在），由调用方如实处置。
    fn fetch(&mut self, out: &mut alloc::vec::Vec<u8>) -> Result<(), libsys::Error>;
}

/// 生产实现：读 `/devices/input/events`（阻塞语义，见 [`libsys::event::EventSourceReader`]）。
impl EventBytes for libsys::event::EventSourceReader {
    fn fetch(&mut self, out: &mut alloc::vec::Vec<u8>) -> Result<(), libsys::Error> {
        self.read_into(out).map(|_| ())
    }
}

pub struct EventSource<R: EventBytes = libsys::event::EventSourceReader> {
    /// 事件字节来源（生产=R 事件节点；测试=可控序列）。
    reader: R,
    /// **keymap 状态机**（Shift/Ctrl 位）。必须跨 `refill` 调用保持——
    /// 否则「Shift 按下」与「Shift 释放」分处两批时状态会丢失。
    keymap: libsys::event::KeymapState,
    /// 与字节流路径**共用**的解码器（转义序列/退格/提交语义的唯一实现）。
    ///
    /// **它同时充当「待取字节」的缓冲区**：转换层一次 `read` 可能产出多个字节
    /// （如 `\x1b[A` 三字节），而 `next_item` 的契约是**一次一个单元**。
    /// 这些字节经 `push_bytes` 进入本字段排队，故无需再造一个缓冲（S13/S28）。
    inner: ByteSource,
}

impl EventSource<libsys::event::EventSourceReader> {
    /// 打开事件节点并构造输入源。
    ///
    /// **无事件节点时如实报错**，不静默回退到键盘直读（S09）——阶段 2 是**双轨**，
    /// 回退决策属调用方（它可以选择用 [`ByteSource`] 那条轨道），
    /// 不应由本类型代为隐瞒。
    pub fn open() -> Result<Self, libsys::Error> {
        Ok(Self {
            reader: libsys::event::EventSourceReader::open()?,
            keymap: libsys::event::KeymapState::default(),
            inner: ByteSource::new(),
        })
    }

    /// 关闭底层 fd（显式）。
    pub fn close(self) -> Result<(), libsys::Error> {
        self.reader.close()
    }
}

impl<R: EventBytes> EventSource<R> {
    /// 用**任意**事件字节来源构造（生产传读取器，测试传可控序列）。
    pub fn from_reader(reader: R) -> Self {
        Self { reader, keymap: libsys::event::KeymapState::default(), inner: ByteSource::new() }
    }
}

impl<R: EventBytes> InputSource for EventSource<R> {
    fn next_item(&mut self) -> InputItem {
        self.inner.next_item()
    }

    fn refill(&mut self) -> bool {
        // **阻塞-唤醒哨兵：这里必须「立刻重试」，不能返回 false**。
        //
        // 内核在事件环空时**挂起本进程**，按键到达后以 `-EAGAIN` 哨兵唤醒，
        // 要求用户态**重试** `read`（`kernel/src/syscall.rs:2297`）。
        // 故 `WouldBlock` 的语义是「**现在就重试**」，而不是「暂无数据」。
        //
        // 【实测缺陷记录】本方法两版都错，方向相反，记录于此以免再犯：
        //   初版：只取一批、空即 `false`，注释误以为「调用方下一轮会重新
        //        进入阻塞 read」。实际哨兵使 read **立即返回**，于是用户态空转。
        //   二版：在 `read_into` 里把哨兵**吞成** `Ok(0)`，再靠本方法有界重试——
        //        吞掉后信号已丢失，仍然空转（宿主 CPU 28%、会话冻结）。
        //
        // **对照证据（S09）**：`evdemo` 与 `evsrcdemo` 走**同一条内核路径**，
        // 唯一差别是前者在 `WouldBlock` 上直接 `continue`（**立刻重试 read**）。
        // 结果：`evdemo` 全程正常（`reads=24 waits=10`）并在退出后把控制权交还 shell；
        // `evsrcdemo` 冻结。故正确形态就是**照搬 evdemo 的重试语义**——
        // 哨兵**不吞、不弃**，原样透传到本方法的循环条件里。
        //
        // **为何仍保留上限**：阻塞是**内核**在 `read` 里做的（挂起本进程），
        // 本方法**无法自行阻塞**；生产实现下每轮都会真阻塞，故上限不会被触及，
        // 而它保证「源确已耗尽」（如宿主测试的有限注入序列）时方法必定返回，
        // 不制造挂死（S20）。
        const MAX_RETRY: usize = 64;
        let mut raw = alloc::vec::Vec::new();
        let mut produced = alloc::vec::Vec::new();
        for _ in 0..MAX_RETRY {
            raw.clear();
            produced.clear();
            match self.reader.fetch(&mut raw) {
                // **哨兵：内核刚挂起又唤醒过本进程，立即重试**（不当作「无数据」）。
                Err(libsys::Error::WouldBlock) => continue,
                // 真错误（含事件节点不存在）：如实返回「补不到」，由调用方处置（S09）。
                Err(_) => return false,
                Ok(()) => {}
            }
            // **关键一步（S13：转换只有一处）**：原始事件记录必须经
            // `libsys::event` 的转换层变成字节，才能交给 `ByteSource` 解码。
            //
            // 【实现缺陷记录】本方法的初版**漏了这一步**——直接把 16 字节的原始记录
            // 推进 `inner`，于是 `ByteSource` 把 scancode 当成 ASCII 逐字节解析，
            // 产出的是 `* * H . .` 这类垃圾而非 `a o A`。该缺陷由本模块的等价性测试
            // 在宿主上当场抓出（真实 QEMU 里表现为「按键完全错乱」）。
            if !raw.is_empty() {
                let _ = libsys::event::decode_into(&mut self.keymap, &raw, &mut produced);
            }
            // 本批无字节（例如整批都是修饰键）：下一批仍可能有数据，继续重试。
            if !produced.is_empty() {
                self.inner.push_bytes(&produced);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use alloc::rc::Rc;
    use core::cell::Cell;

    /// 可控的事件字节来源：按脚本逐批交付，用尽后如实返回「无新字节」。
    ///
    /// 关键：它产出的字节是**事件记录**（16 字节/条），不是 ASCII——
    /// 与真实事件节点同构，故走的是与 QEMU 里**完全相同**的转换路径。
    struct ScriptedEvents {
        batches: Vec<Vec<u8>>,
        served: usize,
        /// 外部可读的「是否已服务完全部批次」。
        ///
        /// **为何需要**：`drain` 必须知道输入**真的**取完了，才能停止。
        /// 初版靠「连续两次 `refill` 返回 `false`」来推断，这是**错的**——
        /// 事件流里「一批全是修饰键 ⇒ 产出 0 字节」是常态，连续两批无字节
        /// **不代表**输入结束。该错误让 dribbled 用例提前退出、丢掉最后一个
        /// `Submit`（实测红灯）。改为**显式**回报耗尽（S09：不靠启发式猜状态）。
        exhausted: Rc<Cell<bool>>,
    }
    impl EventBytes for ScriptedEvents {
        fn fetch(&mut self, out: &mut Vec<u8>) -> Result<(), libsys::Error> {
            if self.served < self.batches.len() {
                out.extend_from_slice(&self.batches[self.served]);
                self.served += 1;
            }
            if self.served >= self.batches.len() {
                self.exhausted.set(true);
            }
            Ok(())
        }
    }

    /// 构造一个脚本化事件源，返回它本身与「耗尽标志」。
    fn scripted(batches: Vec<Vec<u8>>) -> (ScriptedEvents, Rc<Cell<bool>>) {
        let flag = Rc::new(Cell::new(batches.is_empty()));
        (ScriptedEvents { batches, served: 0, exhausted: flag.clone() }, flag)
    }

    /// 一条键事件的线格式（与 `libsys::event` 的布局一致，16 字节小端）。
    const DOWN: u8 = 1;
    const UP: u8 = 2;
    fn ev(kind: u8, e0: bool, code: u16) -> Vec<u8> {
        let flags: u64 = if e0 { 1 } else { 0 };
        let lo = (kind as u64) | (flags << 8) | ((code as u64) << 16);
        let mut v = Vec::new();
        v.extend_from_slice(&lo.to_le_bytes());
        v.extend_from_slice(&0u64.to_le_bytes());
        v
    }
    fn down(code: u16) -> Vec<u8> { ev(DOWN, false, code) }
    fn up(code: u16) -> Vec<u8> { ev(UP, false, code) }

    /// 把事件序列拼成一批交付。
    fn batch(recs: &[Vec<u8>]) -> Vec<u8> {
        let mut v = Vec::new();
        for r in recs { v.extend_from_slice(r); }
        v
    }

    /// 把整条字节流拆成 `InputItem` 序列（反复 refill 直到取完）。
    /// 取完一个输入源的全部单元。
    ///
    /// `exhausted` 由脚本化来源在**服务完全部批次**时置位——用它判断结束，
    /// 而不是靠「连续几次没产出」的启发式（那条路会丢数据，见 `ScriptedEvents`）。
    /// 对 [`ByteSource`] 这类一次性推入的源，传入恒 `true` 的闭包即可：
    /// 它的 `refill` 恒为 `false`，队列空了就是空了。
    fn drain(src: &mut impl InputSource, exhausted: impl Fn() -> bool) -> Vec<InputItem> {
        let mut items = Vec::new();
        let mut guard = 0usize;
        loop {
            guard += 1;
            // 防死循环的硬上界：本测试的输入规模远小于此。
            assert!(guard < 100_000, "drain did not terminate");
            match src.next_item() {
                InputItem::WouldBlock => {
                    let got = src.refill();
                    if !got && exhausted() {
                        // 输入确实取完、且本轮没补到东西：再确认队列真的空了。
                        if matches!(src.next_item(), InputItem::WouldBlock) {
                            break;
                        }
                    }
                }
                other => items.push(other),
            }
        }
        items
    }

    /// **本小点的核心判据（S13 单一语义路径）**：同一串按键，经
    /// 「事件流 + 用户态 keymap」与经「纯字节流」必须产出**完全相同的
    /// `InputItem` 序列**——这是阶段 2 「逐字节等价」的可测形式。
    #[test]
    fn test_event_source_equivalent_to_byte_source() {
        // 一段覆盖多种语义的击键：普通字符、回车、退格、CSI 上箭头、Ctrl-C。
        let events = batch(&[
            down(0x1E), up(0x1E),   // a
            down(0x18), up(0x18),   // o
            down(0x2A),             // Shift 下
            down(0x1E), up(0x1E),   // A
            up(0x2A),               // Shift 上
            down(0x0E), up(0x0E),   // Backspace
            ev(DOWN, true, 0x48),   // E0 上箭头 -> CSI
            down(0x1D),             // Ctrl 下
            down(0x2E), up(0x2E),   // ^C
            up(0x1D),
            down(0x1C), up(0x1C),   // 回车
        ]);
        // 事件路径。
        let (ev_src, done) = scripted(alloc::vec![events.clone()]);
        let mut events_src = EventSource::from_reader(ev_src);
        let got = drain(&mut events_src, || done.get());
        // 字节路径：喂**同一条已转换的字节流**会引入第二套 keymap，
        // 故这里直接喂「期望的字节」——它由内核既有语义决定（见下方断言）。
        //
        // 期望序列（逐项可读，便于失败时定位）：a o A \x08 \x1b[A \x03 \r
        use InputItem::*;
        let want = alloc::vec![
            Char('a'), Char('o'), Char('A'), Backspace,
            HistoryPrev, Interrupt, Submit,
        ];
        assert_eq!(got, want, "event path produced a different item sequence");
    }

    /// 字节路径（[`ByteSource`] + `push_bytes`）对**同一组字节**产出
    /// 与事件路径相同的单元序列——两条路径共用同一解码器（S13）。
    #[test]
    fn test_byte_source_matches_event_source_on_same_bytes() {
        // 事件路径产出以下字节（由内核既有语义决定）：
        //   a o A \x08 \x1b[A \x03 \r
        let bytes: &[u8] = b"aoA\x08\x1b[A\x03\r";
        let mut byte_src = ByteSource::new();
        byte_src.push_bytes(bytes);
        // `ByteSource` 无外部源：队列空即耗尽，故传恒真。
        let via_bytes = drain(&mut byte_src, || true);

        let events = batch(&[
            down(0x1E), up(0x1E), down(0x18), up(0x18),
            down(0x2A), down(0x1E), up(0x1E), up(0x2A),
            down(0x0E), up(0x0E),
            ev(DOWN, true, 0x48),
            down(0x1D), down(0x2E), up(0x2E), up(0x1D),
            down(0x1C), up(0x1C),
        ]);
        let (ev2, done2) = scripted(alloc::vec![events]);
        let mut events_src = EventSource::from_reader(ev2);
        let via_events = drain(&mut events_src, || done2.get());

        assert_eq!(via_events, via_bytes,
                   "event path and byte path must be byte-equivalent");
    }

    /// **跨批到达**（真实事件流的常态）：一条记录一批，结果必须与
    /// 一次性交付**完全相同**——状态机与待取字节缓冲跨 `refill` 保持。
    #[test]
    fn test_event_source_survives_one_record_per_batch() {
        let recs = alloc::vec![
            down(0x2A), down(0x1E), up(0x1E), up(0x2A), down(0x1C), up(0x1C),
        ];
        let one_shot = {
            let (r, d) = scripted(alloc::vec![batch(&recs)]);
            let mut s = EventSource::from_reader(r);
            drain(&mut s, || d.get())
        };
        let dribbled = {
            let (r, d) = scripted(recs.iter().map(|r| r.clone()).collect());
            let mut s = EventSource::from_reader(r);
            drain(&mut s, || d.get())
        };
        use InputItem::*;
        assert_eq!(one_shot, alloc::vec![Char('A'), Submit]);
        assert_eq!(dribbled, one_shot, "one-record-per-batch must be equivalent");
    }

    /// **`refill` 契约**（§6.7）：事件源在「有记录但无字节产出」时
    /// （纯修饰键批次）返回 `false` 是允许的，但**不得**因此丢状态；
    /// 后续批次到来时必须继续正确解码。
    #[test]
    fn test_event_source_keeps_shift_across_empty_yield_batch() {
        let (r, d) = scripted(alloc::vec![
            batch(&[down(0x2A)]),   // Shift 下：解析成功但产 0 字节
            batch(&[down(0x1E), up(0x1E)]), // a -> 应是大写 A
        ]);
        let mut s = EventSource::from_reader(r);
        let items = drain(&mut s, || d.get());
        use InputItem::*;
        assert_eq!(items, alloc::vec![Char('A')],
                   "Shift state must survive a batch that yields no bytes");
    }
}
