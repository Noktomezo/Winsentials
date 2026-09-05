<div align="center">
  <img src="assets/app-thumbnail.png" alt="Winsentials Preview" width="100%" />

  <br/><br/>

  <p align="center">
    <a href="https://github.com/Noktomezo/Winsentials/releases">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/release/Noktomezo/Winsentials.svg?size=sm&amp;mode=dark&amp;theme=slate">
        <img alt="Release" src="https://www.shieldcn.dev/github/release/Noktomezo/Winsentials.svg?size=sm&amp;mode=light&amp;theme=slate">
      </picture>
    </a>
    <a href="https://github.com/Noktomezo/Winsentials/actions">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/ci/Noktomezo/Winsentials.svg?variant=secondary&amp;size=sm&amp;mode=dark&amp;theme=slate">
        <img alt="CI" src="https://www.shieldcn.dev/github/ci/Noktomezo/Winsentials.svg?variant=secondary&amp;size=sm&amp;mode=light&amp;theme=slate">
      </picture>
    </a>
    <a href="https://github.com/Noktomezo/Winsentials/stargazers">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/stars/Noktomezo/Winsentials.svg?variant=secondary&amp;size=sm&amp;mode=dark&amp;theme=slate">
        <img alt="GitHub Stars" src="https://www.shieldcn.dev/github/stars/Noktomezo/Winsentials.svg?variant=secondary&amp;size=sm&amp;mode=light&amp;theme=slate">
      </picture>
    </a>
    <a href="https://github.com/Noktomezo/Winsentials/commits/main">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/last-commit/Noktomezo/Winsentials.svg?variant=secondary&amp;size=sm&amp;mode=dark&amp;theme=slate">
        <img alt="Last commit" src="https://www.shieldcn.dev/github/last-commit/Noktomezo/Winsentials.svg?variant=secondary&amp;size=sm&amp;mode=light&amp;theme=slate">
      </picture>
    </a>
    <a href="AGENTS.md">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/badge/Agent--friendly-AGENTS.md-D97757.svg?variant=secondary&amp;size=sm&amp;mode=dark&amp;theme=slate">
        <img alt="Agent-friendly AGENTS.md" src="https://www.shieldcn.dev/badge/Agent--friendly-AGENTS.md-D97757.svg?variant=secondary&amp;size=sm&amp;mode=light&amp;theme=slate">
      </picture>
    </a>
  </p>

  <p align="center">
    <strong>Winsentials</strong> is an ultra-fast, modern system utility for Windows 10 &amp; 11 engineered in <strong>pure Rust and GPUI</strong>.<br/>
    Fine-tune performance, minimize input latency, declutter the OS, and monitor hardware in real time — with zero WebView overhead and buttery 240+ FPS responsiveness.
  </p>
</div>

---

## ⚡ Highlights

- 🦀 **Pure Rust & GPUI**: Zero Electron/WebView2 runtime overhead. Direct GPU-accelerated rendering powered by Zed's GPUI framework at your monitor's native refresh rate.
- ⏱️ **Instant Cold Start**: Launches in sub-milliseconds with negligible memory footprint in the system tray.
- 🎯 **Gaming Input Engine**: Hardware-level directional SOCD / Counter-strafe neutralization (*SnapKey*), CSRSS I/O priority boost, and real-time Win32 mouse acceleration control.
- 📊 **Real-time Telemetry**: Stepped history graphs and live diagnostics for CPU (per-core load), GPU (dedicated/shared memory and 3D/video engines), RAM, Disks, and Network.
- 🧹 **Interactive Disk Cleaner**: High-speed scanner for system logs, crash dumps, and application caches with accurate freed space calculation.
- 🚀 **Intelligent Startup Manager**: Levenshtein fuzzy search, PE/LNK/VBS/PowerShell icon extraction, and safe Task Scheduler / service control.
- 🔄 **Production Auto-Updater**: Background GitHub Release checks with interactive two-row action toasts, in-toast download progress, and clean app restart.
- 🎨 **Library-Grade UI**: Arclate & Flexoki design systems with Windows 11 Mica/Acrylic material backdrops, continuous spring physics, and zero abrupt styling jumps.

---

## 🎛️ Features

- **🎮 Gaming & Low-Latency Input Engine**  
  Directional SOCD / null-bind neutralization (SnapKey), kernel I/O priority boosting, real-time pointer acceleration control, and low-level keyboard repeat tuning applied instantaneously through native Win32 APIs.

- **🛠️ System, Shell & Network Tuning**  
  Curated Windows 10 and 11 customizations, Explorer navigation decluttering, context menu restoration, and AFD/TCP network stack tuning — backed by interactive action toasts for zero-friction Explorer or system reloads.

- **📈 Real-Time Hardware Telemetry**  
  Low-overhead hardware monitoring with stepped history graphs: per-core CPU topology and load cards, multi-GPU engine tracking (3D, Video, Copy, VRAM), live disk I/O throughput, and network activity.

- **🧹 System Maintenance & Disk Cleanup**  
  Deep multi-category scanner targeting system crash dumps, update caches, diagnostic logs, and temporary application data with granular selection and accurate space reclamation calculations.

- **🚀 Intelligent Startup Manager**  
  Unified inspection of registry autoruns, startup folders, scheduled tasks, and Windows services with PE icon extraction, vendor identification, and instant fuzzy filtering.

---

## 🏗️ Architecture & Philosophy

Winsentials is built as a lightweight, Rust-adapted Feature-Sliced Design (FSD) workspace:

```text
Winsentials/
├── crates/
│   ├── app/           # Main GPUI desktop application (pages, widgets, features, entities)
│   ├── core/          # winsentials-core (library-grade UI components, motion primitives, theme)
│   └── xtask/         # Build pipeline, packaging, and release automation
├── assets/            # Fonts, vector icons, and branding assets
├── installer/         # Inno Setup 6 build scripts and configurations
└── locales/           # English (en) and Russian (ru) internationalization catalogs
```

### Motion-First UI Contract
Every component in `winsentials-core` adheres to a strict visual and behavioral contract:
- **Continuous State Transitions**: Hover, active, focus, and selection states transition through smooth spring physics.
- **Zero Layout Popping**: Dropdown menus, modals, and notifications morph with proportional bounding box interpolation.
- **Reduced Motion Support**: Automatically honors Windows system accessibility settings.

---

## 📥 Installation

### Automated Installer (Recommended)
Download the latest `Winsentials-Setup.exe` from [GitHub Releases](https://github.com/Noktomezo/Winsentials/releases). The installer provides:
- Seamless Windows 11 dark theme integration.
- Start menu & desktop shortcut creation.
- Clean uninstallation via Windows Settings.

### Portable Binary
Download `Winsentials.exe` from the latest release, extract to any directory, and run directly.

---

## 🛠️ Building From Source

### Prerequisites
- **Rust**: 1.85+ (Edition 2024) — install via [rustup.rs](https://rustup.rs)
- **C++ Build Tools**: Visual Studio 2022 C++ build tools (required for Windows APIs)
- **Git**

### Clone & Run

```bash
# Clone the repository
git clone https://github.com/Noktomezo/Winsentials.git
cd Winsentials

# Run debug build
cargo run -p winsentials

# Build optimized release binary
cargo build --release
```

### Quality & Completion Gates

Before submitting code, run the workspace test and verification suites:

```bash
# Code formatting
cargo fmt --all --check

# Compiler checks
cargo check --workspace

# Clippy linter
cargo clippy --workspace -- -D warnings

# Unit & visual GPUI tests
cargo test --workspace

# Code duplication audit (< 7%)
bunx jscpd crates --min-lines 10 --reporters console --summary
```

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>F</kbd> / <kbd>/</kbd> | Focus global search in Startup / Tweaks |
| <kbd>Escape</kbd> | Close active dropdown, dismiss search, or navigate to parent view |
| <kbd>Ctrl</kbd> + <kbd>X</kbd> | Cut selected text in input fields |
| <kbd>Ctrl</kbd> + <kbd>Q</kbd> | Quit application |

---

## 🤝 Contributing & Agents

Contributions are welcome! If you're building with autonomous AI agents or local scripts:
- Review [`AGENTS.md`](AGENTS.md) for architectural boundaries, downward dependency flow, and library promotion rules.
- Maintain documentation integrity and adhere to the Rust completion gate.

---

## 📄 License

Distributed under the terms of the project repository. See individual crate manifests for dependency licensing.
