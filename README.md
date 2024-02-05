This is a utility showing available disk space, available as a command-line tool, a system tray app for Windows, and a BitBar plugin for macOS.

Minimum Rust version: 1.32.0.

# Installation

1. Run one of the following commands depending on which interface you'd like to install:
    * For the command-line tool: `cargo install --git=https://github.com/fenhl/diskspace --branch=main --bin=diskspace`
    * For the system tray app: `cargo install --git=https://github.com/fenhl/diskspace --branch=main --bin=systray-diskspace`
    * For the BitBar plugin: `cargo install --git=https://github.com/fenhl/diskspace --branch=main --bin=bitbar-diskspace`
2. If you use [`cargo-update`](https://crates.io/crates/cargo-update) to update your installed crates, make sure to run `cargo install-update-config diskspace --respect-binaries` so `cargo-update` won't attempt to install additional interfaces when updating `diskspace`.
