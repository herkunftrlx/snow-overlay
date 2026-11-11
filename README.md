# ❄️ snow-overlay (`itsnowsinside`)

> A blazingly fast, lightweight, anti-cheat safe 144Hz desktop snowfall overlay for Windows, written in 100% pure idiomatic Rust.

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?style=flat&logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows-blue.svg?style=flat&logo=windows)](https://microsoft.com/windows)
[![Performance](https://img.shields.io/badge/FPS-144Hz%20Locked-brightgreen.svg)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

`snow-overlay` brings gentle, calming, cinematic falling snow directly to your Windows desktop. It floats on top of all windows (including IDEs, browsers, and borderless windowed games like Rocket League) without taking focus or intercepting clicks.

---

## ✨ Features

- **🎯 100% Click-Through (`WS_EX_TRANSPARENT`):** Mouse clicks and keyboard strokes pass straight through to whatever window is underneath. Zero input lag, zero mouse obstruction.
- **🛡️ Anti-Cheat Safe:** Zero DLL injection, zero D3D/Vulkan hooking, zero process memory manipulation. Completely external to games—runs purely within Windows Desktop Window Manager (DWM).
- **🚀 144Hz E-Sports Grade Fluidity:** Built with high-precision `timeBeginPeriod(1)` 1ms system timers and delta-time continuous sub-pixel physics.
- **⚡ Ultra Lightweight:** 
  - Binary size: **~145 KB**
  - RAM usage: **~10 MB**
  - CPU usage: **< 0.1%**
- **📊 Real-Time On-Screen FPS Counter:** Displays a clean, toggleable green FPS readout at the top-right corner.
- **🖥️ Multi-Monitor Aware:** Toggle between Primary Monitor (144Hz lock) and Full Virtual Desktop across multi-monitors on the fly.
- **🌬️ Natural Wind Sway:** Realistic particle depth, varied fall speeds, and gentle sine-wave horizontal drift.

---

## ⌨️ Global Hotkeys

| Hotkey | Action |
|---|---|
| `Ctrl + Alt + S` | **Toggle Snowfall** (Pause / Resume) |
| `Ctrl + Alt + F` | **Toggle FPS Counter** (Show / Hide) |
| `Ctrl + Alt + M` | **Toggle Monitor Mode** (Primary 144Hz vs. All Monitors) |
| `Ctrl + Alt + Q` | **Quit** application |

*(You can also press `Ctrl + C` in the console to exit cleanly).*

---

## 🛠️ Building & Running

### Prerequisites
- [Rust](https://rustup.rs/) (1.80+ recommended)
- Windows 10 / 11

### Quick Start
```powershell
# Clone the repository
git clone https://github.com/<your-username>/snow-overlay.git
cd snow-overlay

# Build with optimizations
cargo build --release

# Run directly
.\target\release\snow-overlay.exe
```

Or simply double-click `run_snow.bat`.

---

## 🎮 Gaming Notes (Rocket League, Apex, etc.)
To see the snow over games:
- Set your game's display mode to **Borderless Windowed**.
- In Exclusive Fullscreen, Windows bypasses DWM desktop composition. Borderless Windowed allows DWM to composite the transparent snow layer directly over your gameplay.

---

## 📜 License
Licensed under the [MIT License](LICENSE).
