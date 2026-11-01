<p align="center">
  <img src="docs/logo.png" alt="Musubi Hub" width="128">
</p>

<h1 align="center">Musubi Hub</h1>

<p align="center">
  See and control your mouse, macros and webcam without the heavy vendor software.<br>
  Free and open source. Light, private, no account.
</p>

<p align="center">
  <a href="https://github.com/WhiteRecon177/musubi-hub/releases/latest"><b>Download for Windows</b></a>
  &nbsp;·&nbsp; <a href="#getting-started">Getting started</a>
  &nbsp;·&nbsp; <a href="#privacy">Privacy</a>
  &nbsp;·&nbsp; <a href="CONTRIBUTING.md">Contribute</a>
</p>

<p align="center">
  <img src="docs/screenshots/mouse-mono.png" alt="Musubi Hub, Mouse tab" width="720">
</p>

## Why Musubi Hub

- **Everything in one small window.** Battery, DPI and polling rate of your mouse, macros and button remapping, webcam controls with saved profiles.
- **Light.** A few MB installer, no background services. Closing the window can keep it quietly in the system tray.
- **Live.** Plug or unplug the cable and the state updates instantly.
- **Private.** No account, no telemetry, no analytics. The only thing the app asks the internet is whether a newer version exists, and you can turn that off.
- **Yours.** Three themes (Monochrome, Ocean Glass, Crimson) and 8 languages: English, Italiano, Español, Deutsch, Français, Português, Русский, 中文.
- **Updates itself.** A small notice tells you when a new version is out, with a one-click update.

## Screenshots

<p align="center">
  <img src="docs/screenshots/macro-mono.png" alt="Macro tab" width="360">
  <img src="docs/screenshots/webcam-mono.png" alt="Webcam tab" width="360">
</p>
<p align="center">
  <img src="docs/screenshots/mouse-ocean.png" alt="Ocean Glass theme" width="360">
  <img src="docs/screenshots/mouse-crimson.png" alt="Crimson theme" width="360">
</p>

## Download

All files are on the [releases page](https://github.com/WhiteRecon177/musubi-hub/releases/latest).

| | |
|---|---|
| **Windows installer** · `Musubi Hub_x.y.z_x64-setup.exe` | Installs for your user only, no administrator rights needed. |

Requirements: Windows 10 or 11, 64-bit, with the Microsoft Edge WebView2 Runtime (already part of Windows 11).

> **Note:** the installer is not code-signed yet, so Windows SmartScreen may warn you the first time: choose *More info → Run anyway*. The installer is built by GitHub Actions from this source.

## Getting started

1. Run the installer and start **Musubi Hub**.
2. **Mouse** tab: pick your device, see battery, DPI and polling rate. Changes are applied to the mouse.
3. **Macro** tab: record and play macros, remap buttons.
4. **Webcam** tab: adjust brightness, contrast, focus and more, and save profiles.
5. Open the gear in the top right for language, theme, start with Windows and updates.

## Supported mice

| Brand | Status |
|---|---|
| Logitech (HID++ 2.0, LIGHTSPEED) | Working: DPI, polling rate, battery |
| Attack Shark, Pulsar, Lamzu, Glorious, Microsoft, Roccat, SteelSeries | Protocols documented, drivers not written yet |

Drivers are **read-only by design** where the protocol is not fully known: the app never sends firmware, DFU, reset, pairing or flash-save commands. The research notes (in Italian, unverified, with sources) are in [`docs/PROTOCOLLI.md`](docs/PROTOCOLLI.md).

Mouse support is the main thing missing, and you can help even without writing code: see [CONTRIBUTING.md](CONTRIBUTING.md).

## Privacy

- No account, no telemetry, no analytics. Nothing you do in the app is sent anywhere.
- The only network request is the update check to GitHub Releases (Settings → "Check for updates at start").
- Macros and settings stay on your PC. Keys and mouse buttons are read only while you record a macro or use a remapped button, and never leave the app.
- Because it can read global input (needed for macros and button remapping), some antivirus programs may flag it the first time. The code is open: you can read it or build it yourself.

## Please read

- **Games with anti-cheat** (Valorant/Vanguard, FACEIT, EAC, BattlEye...) may treat macros and remapped buttons as cheating and ban your account. Do not use the Macro tab in online games. You do it at your own risk.
- **Use at your own risk.** The drivers talk to your devices over standard HID. Writing settings is limited to documented commands, but no warranty is given (see the licence).

## Building from source

Requirements: Windows 10/11, [Node.js](https://nodejs.org), [Rust](https://rustup.rs) and the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev      # run with hot reload
npm run tauri build -- --bundles nsis   # build the installer
```

Built with [Tauri 2](https://tauri.app) (Rust) + React + Vite + TypeScript. How releases and automatic updates work: [`docs/UPDATES.md`](docs/UPDATES.md).

## Contributing

Bug reports, mouse data, translations and pull requests are welcome: see [CONTRIBUTING.md](CONTRIBUTING.md).

## Trademarks and independence

Musubi Hub is an independent open-source project. It is **not affiliated with, endorsed by or sponsored by** Logitech, Razer, Corsair, SteelSeries, Microsoft, Glorious, Pulsar, Lamzu, Attack Shark or any other brand. All product names, logos and brands are property of their owners and are used only to say which devices are compatible. The interface and logo are original work.

Device support comes from the public HID standard and from the interoperability work of the community. Only facts (ids, byte frames) are used, never code, from projects with a copyleft licence; see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
