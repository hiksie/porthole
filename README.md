# Porthole

Porthole is a small desktop app for sharing folders with other devices on your local network. Pick the folders you want to share, start the built-in server, and open the generated link (or scan the QR code) from any phone, tablet, or computer on the same network to browse, download, and upload files — no cloud, no accounts.

## Features

- **Share any number of folders** — add or remove them from the desktop app at any time.
- **Built-in web server** — served over your LAN, browsable from any device with a browser.
- **QR code sharing** — scan it with a phone camera instead of typing an IP address.
- **Upload & download** — the web UI lets connected devices browse folders and transfer files in both directions.
- **No accounts, no cloud** — everything stays on your local network.
- **Cross-platform GUI** — built with [`iced`](https://github.com/iced-rs/iced).

## Getting started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, edition 2024)

### Build & run

```sh
git clone git@github.com:hiksie/porthole.git
cd porthole
cargo build --release
cargo run --release
```