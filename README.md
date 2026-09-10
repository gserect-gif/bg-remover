# BG Remover

A local, offline Windows desktop app that removes image backgrounds using
an on-device AI model (IS-Net general-use, Apache-2.0). No cloud calls, no
accounts, no telemetry. Import → remove background → export transparent PNG.

## Architecture

- **Backend**: Rust + Tauri 2 — owns the AI model, image decode/encode, and
  all file I/O. The ONNX Runtime session is loaded once at app startup and
  kept alive for the app's lifetime (see `src-tauri/src/pipeline.rs`).
- **AI model**: [IS-Net general-use](https://github.com/xuebinqin/DIS)
  (Apache-2.0), running via [`ort`](https://ort.pyke.io) (Rust ONNX Runtime
  bindings) on CPU — no GPU required.
- **Frontend**: React + TypeScript, talking to the backend via Tauri's
  `invoke` bridge. No web server, no network calls.

## Getting a real Windows `.exe`

You have two options. Both produce the same genuine, installable app —
pick whichever is easier for you.

### Option A — Automatic build via GitHub Actions (recommended, no local setup)

1. Create a new (private or public) GitHub repo and push this project to it:
   ```
   git init
   git add .
   git commit -m "Initial commit"
   git branch -M main
   git remote add origin https://github.com/YOUR_USERNAME/bg-remover.git
   git push -u origin main
   ```
2. Go to the repo's **Actions** tab on GitHub. The `Build Windows App`
   workflow runs automatically on every push to `main` (or trigger it
   manually via "Run workflow").
3. When it finishes (~10–15 minutes, mostly downloading the model and
   compiling Rust), open the finished run and download the
   **`BG-Remover-Windows-Installer`** artifact — that's your real `.exe`
   installer. There's also a **`BG-Remover-Windows-Raw-Exe`** artifact if
   you just want the standalone binary without an installer wrapper.

No Rust, Node, or Windows machine required on your end — GitHub's own
Windows runners do the compiling.

### Option B — Build locally on your Windows machine

Prerequisites:
- [Rust](https://rustup.rs) (stable toolchain)
- [Node.js](https://nodejs.org) 20+
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
  with the "Desktop development with C++" workload (required by Tauri on
  Windows)

Steps:
```powershell
# 1. Install frontend deps
npm install

# 2. Download the AI model (one-time, ~178MB)
./scripts/fetch-model.ps1

# 3. Install the Tauri CLI if you don't have it
npm install -g @tauri-apps/cli

# 4. Build the release app
npx tauri build
```

The installer lands in `src-tauri/target/release/bundle/nsis/`, and the
raw `.exe` in `src-tauri/target/release/bg-remover.exe`.

To iterate during development instead of doing a full release build:
```powershell
npx tauri dev
```

## Project layout

```
src/                      React frontend
  App.tsx                 State machine + native drag-drop wiring
  components/              UI pieces (drop zone, preview, control rail)
  lib/backend.ts           Typed wrapper around Tauri commands
src-tauri/
  src/
    main.rs                Tauri entry point, command registration
    pipeline.rs             Full image pipeline: decode → infer → refine → composite
    inference.rs             ONNX Runtime session + preprocessing
    error.rs                 Error types shown to the user
  models/                  AI model lives here (fetched, not committed)
  tauri.conf.json           App/window/bundle configuration
scripts/fetch-model.ps1    Downloads the IS-Net model
.github/workflows/         CI that builds the Windows installer automatically
```

## Notes on quality decisions

- **Mask refinement** (`pipeline.rs::refine_mask`) removes small disconnected
  fragments ("crumbs") via connected-component filtering, then applies
  edge-aware smoothing that only touches pixels near actual mask boundaries
  — flat foreground/background regions are left bit-for-bit untouched, so
  fine detail isn't blurred away just to clean up edges.
- **Resolution**: inference runs at a capped resolution for CPU performance,
  but the resulting mask is bilinearly upsampled back to the *original*
  image dimensions before compositing, so exported PNGs always match the
  input's true resolution.
- **Model reuse**: the ONNX session is loaded once (`ensure_engine_loaded`)
  and reused for every image in the session — no repeated model-init cost.

## License note

The bundled IS-Net general-use model weights are Apache-2.0 licensed
(original work: Qin et al., "Highly Accurate Dichotomous Image
Segmentation", ECCV 2022 — `xuebinqin/DIS`). Redistribution with this app
is permitted under that license.
