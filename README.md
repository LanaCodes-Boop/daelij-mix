# Daelij MiX

An audio mixer for Windows. Put your apps on **Game**, **Chat**, **Media** or **Aux** and control each group with its own fader.

## Try it

1. Open **[Releases](../../releases)** and download the newest `..._x64-setup.exe`.
2. Run it. Windows may show *"Windows protected your PC"* because the app is not signed yet: click **More info → Run anyway**.
3. Play some sound (music, a game, Discord) and open the **Applications** tab: your apps show up there with live meters.

## Stream mix (Mix B) setup

Mix B is a second mix for your stream, separate from what you hear.

1. Install the free **VB-Cable** (vb-audio.com/Cable) and restart the PC.
2. In Daelij MiX open the device menu (top right). Under Mix B, "CABLE Input" should be first.
3. In OBS: Sources → + → **Audio Input Capture** → **CABLE Output**. Mute OBS desktop audio so nothing plays twice.

Needs Windows 11 (or Windows 10 build 20348+). Daelij MiX records each app separately (Windows process loopback), so only one virtual cable is needed.

## What works in this test version

- Game / Chat / Media / Aux faders change the real Windows volume of the apps on them (Mix A, your headphones/speakers).
- Mix B plays a separate stream mix on a stream device, with a priority list: if the first device fails or is unplugged, the next one is used, and it switches back when the first returns.
- Live meters per app, per bus and for Mix B.
- Drag apps between buses, per-app volume and mute. New apps are placed automatically and marked "Auto" until you choose.
- Mix A / Mix B focus, routing matrix, groups, aliases.
- Scenes: checked before loading (a damaged scene is refused), gains glide over 50 ms, one undo step. Unsaved-changes dot.
- Undo/redo for the last 100 actions (Ctrl+Z / Ctrl+Y), R resets peak holds.
- Compact / Standard / Detailed density, inspector panel.

When you close Daelij MiX, every app goes back to 100% volume.

## Known limits

- If you turn a source fully down for your headphones (Mix A), it also goes quiet on the stream: Windows applies the app volume before Daelij MiX can record it.
- The effects (EQ, compressor, gate, noise reduction) are still a preview.
- The app is not code-signed.

Design ideas (process tree, scenes, fallback, 100-step undo) partly come from the Mixdeck design.

## Build it yourself

Needs Node 20+ and Rust (stable) on Windows.

```
npm ci
npm run tauri build
```

`npm run dev` starts the web demo (demo signal) in a browser.

Built with Svelte, Vite and Tauri 2. Windows audio through Core Audio (WASAPI sessions, process loopback via the `wasapi` crate).
