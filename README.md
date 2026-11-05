# Daelij MiX

An audio mixer for Windows. Put your apps on **Game**, **Chat**, **Media** or **Aux** and control each group with its own fader.

## Try it

1. Open **[Releases](../../releases)** and download the newest `..._x64-setup.exe`.
2. Run it. Windows may show *"Windows protected your PC"* because the app is not signed yet: click **More info → Run anyway**.
3. Play some sound (music, a game, Discord) and open the **Applications** tab: your apps show up there with live meters.

## What works in this test version

- Game / Chat / Media / Aux faders change the real Windows volume of the apps on them (via Mix A, your headphones/speakers).
- Live meters per app and per bus.
- Drag apps between buses, per-app volume and mute.
- Mix A / Mix B focus (the source faders show the sends to the selected mix), routing matrix, groups, aliases.
- Scenes with unsaved-changes dot, undo/redo (Ctrl+Z / Ctrl+Y), R resets peak holds.
- Compact / Standard / Detailed density, inspector panel.

When you close Daelij MiX, every app goes back to 100% volume.

## Not real yet

- **Mix B (stream)** and the **effects** (EQ, compressor, gate, noise reduction) are a preview. Sending a separate mix to OBS and processing sound need a virtual audio driver, which is not built yet.
- The app is not code-signed.

## Build it yourself

Needs Node 20+ and Rust (stable) on Windows.

```
npm ci
npm run tauri build
```

`npm run dev` starts the web demo (demo signal) in a browser.

Built with Svelte, Vite and Tauri 2. Windows audio through Core Audio (WASAPI session API).
