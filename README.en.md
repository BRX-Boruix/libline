# libline

BORUIX's **user-space line editing library**: it turns a sequence of input items into one editable line of text.

[简体中文](README.md)

## Features

| Capability | Details |
| --- | --- |
| Character editing | Typing, backspace, delete, moving the cursor left and right |
| Movement within a line | Jump to the start or end of the line, move by word |
| History | Page up and down through previously entered lines |
| Tab completion | Complete by candidate prefix |
| Echo suppression | Input is not displayed (used for password entry) |

## Usage

Referenced as a dependency:

```toml
[dependencies]
libline = { path = "../libline" }
```

A caller may use all of the capabilities or only a subset:

| Caller | Capabilities used |
| --- | --- |
| [`shell`](https://github.com/BRX-Boruix/shell) | All — history, completion, cursor editing, redraw |
| [`login`](https://github.com/BRX-Boruix/login) | Echo suppression only |

## Building

```bash
cargo test      # runs the editing core's tests on the host
```

## Layout

```
libline/src/
├── lib.rs      # module exports
├── editor.rs   # the editing core
└── source.rs   # input sources and the byte stream implementation
```

## Related projects

- [`shell`](https://github.com/BRX-Boruix/shell) — uses all the editing capabilities
- [`login`](https://github.com/BRX-Boruix/login) — uses echo suppression only
- [`libsys`](https://github.com/BRX-Boruix/libsys) — provides the system call wrappers

## License

MIT License, copyright Yang Borui. See [LICENSE](LICENSE).
