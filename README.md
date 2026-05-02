<div align="center">

<img src="docs/icon.svg" width="250" alt="EliteCleaner logo">

<br>
<h3>EliteCleaner</h3>
<h6>Desktop system cleaner with advanced trace cleaning tools</h6>

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue?style=flat-square&logo=tauri)](https://tauri.app/)
[![License](https://img.shields.io/github/license/neverjoskiy/eliteclean?color=green&style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Windows-10%2F11-lightblue?style=flat-square&logo=windows)](https://www.microsoft.com/windows)
<br>
[![Build](https://img.shields.io/badge/build-Release-brightgreen?style=flat-square)]
[![WebView2](https://img.shields.io/badge/WebView2-Required-blue?style=flat-square)]

English • [Русский](README_ru.md)

</div>

<br>

## What is EliteCleaner?

**EliteCleaner** is a desktop application built with **Rust + Tauri v2** for deep system trace cleaning. The interface is built with vanilla HTML/CSS/JS — no external frameworks. Frontend communicates with the backend via `window.__TAURI__.core.invoke()`.

**Designed for:**

- 🛠 System maintenance and cleanup
- 🔍 Digital forensics and trace analysis
- 🔐 Privacy-conscious users
- 📡 Windows system administration

## Why EliteCleaner?

Traditional Windows cleanup means using separate tools for each task — registry cleaners, temp file removers, network reset tools, all opened **in separate windows**.

You constantly **switch** between different utilities, and the tools are **not linked** together.

#### **EliteCleaner solves this!**
- 🔘 Everything is in one place
- 🔗 All tools are connected
- 💻 Unified workflow

![screenshot](docs/screenshot.png)

## Features

### Available now

| Feature | Description |
|---|---|
| 🔍 System Scanner | Analyze system by categories with size calculation |
| 🎯 Selective Cleaning | Clean found files selectively |
| 📊 Animated Results | Animated odometer for scan results |

### Tools

| Tool | Description |
|---|---|
| `USN Journal` | Delete and recreate NTFS change journal |
| `Trace Cleaner` | Shellbag, Explorer, Prefetch, Minidump |
| `Memory Wiper` | Find and wipe strings in javaw.exe process memory |
| `Folder Simulation` | Launch external simulation tool |
| `Global Cleanup` | Event Log, Prefetch, Amcache, Jump Lists, Browser History, Temp |

### Network

| Feature | Description |
|---|---|
| 🌐 DNS Cache Reset | Clear DNS resolver cache |
| 🔄 NetBIOS Reset | Clean NetBIOS cache |
| 🌍 Full Network Reset | Reset Winsock, IP, IPv6 |

### System & Privacy

| Feature | Description |
|---|---|
| 🧹 Registry Cleaner | RunMRU, RecentDocs, UserAssist, TypedPaths |
| 💾 Memory Dumps | Remove Minidump, MEMORY.DMP, CrashDumps |
| 🔧 Windows Update Cache | Clean update cache |
| 🖼 Thumbnail Cache | Clear thumbnail cache |
| 📋 Clipboard | Clipboard + history cleanup (Win10+) |
| 🔎 Search History | WordWheelQuery cleanup |
| 🚀 Run History | RunMRU cleanup |

## Build

### Prerequisites

| Dependency | Minimum version |
|---|---|
| **Rust** | 1.70+ |
| **WebView2** | Runtime (built-in on Win11) |
| **Node.js** | Optional (for frontend development) |

### Build from source

```bash
git clone https://github.com/neverjoskiy/eliteclean.git
cd eliteclean

# Development mode
cd src-tauri
cargo run

# Release build
cargo build --release
```

> Binary location: `src-tauri/target/release/elite-cleaner.exe`

## Project Structure

```
eliteclean/
├── static/                  # Frontend
│   ├── index.html
│   ├── css/styles.css
│   └── js/app.js
├── scripts/                 # Batch cleanup scripts
│   ├── вирус.bat            # Remove USN journal
│   ├── не вирус.bat         # Create USN journal
│   └── винлокер.bat         # Trace cleanup (admin)
├── src-tauri/
│   └── src/
│       ├── bin/main.rs      # Entry point, command registration
│       ├── commands.rs      # Tauri commands (invoke from JS)
│       ├── services.rs      # Business logic
│       ├── memory.rs        # javaw.exe memory operations
│       ├── models.rs        # Data structures (serde)
│       ├── state.rs         # AppState global state
│       └── utils.rs         # Paths, logging
├── docs/                    # Documentation assets
├── release/                 # Ready-to-use build
├── CHANGELOG.md
└── README.md
```

## Requirements

- **OS:** Windows 10 / 11
- **Runtime:** [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (built-in on Win11)
- **Build:** [Rust](https://rustup.rs) 1.70+
- **Rights:** Some tools require administrator privileges

## Changelog

See [CHANGELOG.md](./CHANGELOG.md)

## Contributing

Contributions are **welcome and encouraged**.

Feel free to open an issue or submit a pull request.

## License

Distributed under the terms described in [LICENSE](LICENSE).

---

<div align="center">
<sub>built with Rust 🦀 + Tauri ⚡</sub>
</div>