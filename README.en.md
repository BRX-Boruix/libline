# libline

BORUIX's user-space line editing library, turning a sequence of input items into one editable line of text.

[简体中文](README.md)

## Features

- Character editing: typing, backspace, delete, moving the cursor left and right
- Movement within a line: jump to the start or end, move by word
- History: page up and down through previously entered lines
- Tab completion: complete by candidate prefix
- Echo suppression: input is not displayed, used for password entry

## Usage

Referenced as a dependency:

```toml
[dependencies]
libline = { path = "../libline" }
```

A caller may use all of the capabilities or only a subset. [`shell`](https://github.com/BRX-Boruix/shell) uses all of them; [`login`](https://github.com/BRX-Boruix/login) uses echo suppression only.

## Building

```bash
cargo test      # runs the editing core's tests on the host
```

## Repository layout

- `src/lib.rs` — module exports
- `src/editor.rs` — the editing core
- `src/source.rs` — input sources and the byte stream implementation

## Related projects

- [`shell`](https://github.com/BRX-Boruix/shell) — uses all the editing capabilities
- [`login`](https://github.com/BRX-Boruix/login) — uses echo suppression only
- [`libsys`](https://github.com/BRX-Boruix/libsys) — provides the system call wrappers

## License

MIT License, copyright Yang Borui. See [LICENSE](LICENSE).
