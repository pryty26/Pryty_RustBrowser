Discord:
https://discord.gg/CYNQ25XP8B



```markdown
# Pryty RustBrowser

[![Crates.io](https://img.shields.io/crates/v/pryty-rustbrowser.svg)](https://crates.io/crates/pryty-rustbrowser)
[![Docs.rs](https://img.shields.io/docsrs/pryty-rustbrowser.svg)](https://docs.rs/pryty-rustbrowser)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

**Production-ready browser API bindings for Dioxus.**  
One-line hooks for camera, microphone recording, clipboard, and localStorage — with full TypeScript-level safety and complete error handling.

## ✨ Features

- 🎙️ **Audio Recording** — Multi-quality presets (low to lossless), pause/resume, time slices, auto-stop timer
- 📹 **Camera** — Resolution/frame rate presets (480p to 1080p), track lifecycle management
- 📋 **Clipboard** — Async read/write with proper error propagation
- 💾 **Storage** — Type-safe localStorage with automatic JSON serialization (powered by `serde`)
- 🪝 **Dioxus Hooks** — `use_recording`, `use_camera`, `use_clipboard`, `use_storage` — integrate in one line
- 🧪 **Error Handling** — Rich error types for every failure mode
- 🔒 **Production Ready** — Proper cleanup (tracks, timers, closures), no leaks

## 📦 Installation

```bash
cargo add pryty-rustbrowser
```

Also add `serde` for storage (derive features as needed):

```bash
cargo add serde --features derive
```

## 🚀 Quick Start

```rust
use dioxus::prelude::*;
use pryty_rustbrowser::{use_recording, use_camera, use_clipboard, use_storage, AudioQualityConfig};

#[component]
fn App() -> Element {
    // One line — fully typed reactive state
    let recording = use_recording();
    let camera = use_camera();
    let clipboard = use_clipboard();
    let (username, set_username) = use_storage("username".to_string(), "Anonymous".to_string());

    rsx! {
        div { class: "p-4 space-y-4",

            // ---------- Audio Recording ----------
            div {
                h3 { "🎙️ Audio Recording" }
                p { "State: {recording.state:?}" }
                div { class: "flex gap-2",
                    button {
                        onclick: move |_| recording.start(),
                        disabled: recording.is_busy(),
                        "Start"
                    }
                    button {
                        onclick: move |_| recording.start_with_quality(AudioQualityConfig::high()),
                        disabled: recording.is_busy(),
                        "Start (High Quality)"
                    }
                    button {
                        onclick: move |_| recording.pause(),
                        disabled: !recording.is_active(),
                        "Pause"
                    }
                    button {
                        onclick: move |_| recording.resume(),
                        disabled: !recording.is_paused(),
                        "Resume"
                    }
                    button {
                        onclick: move |_| recording.stop(),
                        disabled: !recording.is_active() && !recording.is_paused(),
                        "Stop"
                    }
                }
                if let Some(data) = recording.data() {
                    p { "Recorded: {} bytes", data.len() }
                }
            }

            // ---------- Camera ----------
            div {
                h3 { "📷 Camera" }
                p { "State: {camera.state:?}" }
                div { class: "flex gap-2",
                    button {
                        onclick: move |_| camera.start(),
                        disabled: camera.is_active(),
                        "Start Camera"
                    }
                    button {
                        onclick: move |_| camera.start_with_quality(CameraQualityConfig::hd()),
                        disabled: camera.is_active(),
                        "Start (HD)"
                    }
                    button {
                        onclick: move |_| camera.stop(),
                        disabled: !camera.is_active(),
                        "Stop Camera"
                    }
                }
                // Display camera preview
                if let Some(stream) = camera.stream() {
                    video {
                        src_object: Some(stream),
                        autoplay: true,
                        muted: true,
                        class: "mt-2 w-96 rounded-lg border"
                    }
                }
            }

            // ---------- Clipboard & Storage ----------
            div {
                h3 { "📋 Clipboard + Storage" }
                input {
                    value: "{username}",
                    oninput: move |evt| set_username(evt.value()),
                    class: "border px-2 py-1"
                }
                button {
                    onclick: move |_| clipboard.write(&username)(),
                    "Copy to Clipboard"
                }
                button {
                    onclick: move |_| spawn(async move {
                        if let Ok(Some(text)) = clipboard.read().await {
                            println!("Read from clipboard: {text}");
                        }
                    }),
                    "Read from Clipboard"
                }
            }
        }
    }
}

fn main() {
    dioxus::launch(App);
}
```

## 🧩 API Reference

### 🎙️ `use_recording() -> Recording`

| Field/Method | Type | Description |
|--------------|------|-------------|
| `start()` | `Callback<()>` | Start recording with default quality |
| `start_with_quality(Quality)` | `Callback<Quality>` | Start with preset (low/normal/high/studio/lossless) |
| `pause()` | `Callback<()>` | Pause recording |
| `resume()` | `Callback<()>` | Resume recording |
| `stop()` | `Callback<()>` | Stop and finalize audio data |
| `data()` | `Signal<Option<Vec<u8>>>` | Recorded audio bytes (WebM/Opus or PCM) |
| `state()` | `Signal<RecordingState>` | Idle / Starting / Recording / Paused / Stopping / Error |
| `last_error()` | `Signal<Option<String>>` | Last error message |
| `is_active()` | `bool` | Returns `true` if currently recording |
| `is_paused()` | `bool` | Returns `true` if paused |
| `is_busy()` | `bool` | Returns `true` during starting/stopping |

**Quality Presets:**

| Method | Sample Rate | Channels | Bitrate |
|--------|-------------|----------|---------|
| `AudioQualityConfig::low()` | 22.05 kHz | 1 (mono) | 64 kbps |
| `normal()` | 44.1 kHz | 1 | 128 kbps |
| `high()` | 48 kHz | 2 (stereo) | 192 kbps |
| `studio()` | 96 kHz | 2 | 320 kbps |
| `lossless()` | 96 kHz | 2 | uncompressed PCM |

### 📷 `use_camera() -> Camera`

| Field/Method | Type | Description |
|--------------|------|-------------|
| `start()` | `Callback<()>` | Start camera with default constraints |
| `start_with_quality(Quality)` | `Callback<Quality>` | Start with resolution/frame rate preset |
| `stop()` | `Callback<()>` | Stop camera and release tracks |
| `stream()` | `Signal<Option<MediaStream>>` | Raw WebRTC stream (for `<video>` preview) |
| `state()` | `Signal<CameraState>` | Idle / Starting / Active / Stopping / Error |
| `is_active()` | `bool` | Returns `true` if camera streaming |
| `is_busy()` | `bool` | Returns `true` during start/stop |

### 📋 `use_clipboard() -> Clipboard`

| Method | Description |
|--------|-------------|
| `write(text)(&self)` | Fire-and-forget write (async inside) |
| `write_async(text).await` | Async write with `Result` |
| `read().await` | Async read → `Result<Option<String>>` |

### 💾 `use_storage<T>(key, default) -> (Signal<T>, Callback<T>)`

**Requirements:** `T: Serialize + DeserializeOwned + Clone`

```rust
let (count, set_count) = use_storage("counter".to_string(), 0);
set_count(count() + 1);
```

Also available: `read_storage::<T>(key)` and `write_storage(key, &value)` for one-off operations.

## ⚠️ Error Handling

Every operation returns rich errors. Example:

```rust
match camera.start().await {
    Ok(_) => println!("Camera ready"),
    Err(CameraError::GetUserMediaFailed(msg)) => {
        eprintln!("User denied permission or no camera: {msg}")
    }
    Err(e) => eprintln!("Other error: {e}"),
}
```

All error types implement `std::fmt::Display` and `Debug`.

## 🔧 Browser Compatibility

| API | Requires |
|-----|----------|
| Audio Recording | `MediaRecorder`, `getUserMedia` |
| Camera | `getUserMedia`, `<video>` element |
| Clipboard | `navigator.clipboard` (HTTPS or localhost) |
| Storage | `localStorage` |

All gracefully return errors when APIs unavailable.

## 🧪 Example Project

Clone and run the demo:

```bash
git clone https://github.com/YOUR_USERNAME/pryty-rustbrowser
cd pryty-rustbrowser
cargo run --example demo
```

Or use `dx serve` for web development.

## 🤝 Contributing

PRs welcome! Please ensure:
- All public APIs have doc comments
- Error types cover all failure paths
- No `unwrap()` / `expect()` in library code (use `?` or proper handling)

## 📄 License

Dual-licensed under MIT or Apache-2.0 at your option.

---

**Built for Dioxus 0.7+** — works with WebAssembly, desktop, and mobile backends.
```

