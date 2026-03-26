
# pryty-rustbrowser

Browser API hooks for Dioxus web apps.

## Install

```bash
cargo add pryty-rustbrowser
```

## What this crate provides

This crate exposes four hook groups:

- Audio recording
- Camera stream control
- Typed localStorage state
- Clipboard read/write

---

## API Overview

### Recording

#### `use_recording() -> Recording`

Creates a recording controller.

`Recording` fields:

- `start: Callback<()>`
- `stop: Callback<()>`
- `data: Signal<Option<Vec<u8>>>`
- `state: Signal<RecordingState>`
- `last_error: Signal<Option<String>>`

`Recording` methods:

- `is_active(&self) -> bool`  
  Returns `true` only while actual recording is active.
- `is_busy(&self) -> bool`  
  Returns `true` while transitioning (`Starting` or `Stopping`).

`RecordingState`:

- `Idle`
- `Starting`
- `Recording`
- `Stopping`
- `Error(String)`

`RecordingError` variants:

- `WindowUnavailable`
- `MediaDevicesUnavailable`
- `GetUserMediaFailed(String)`
- `CastMediaStreamFailed`
- `RecorderCreateFailed(String)`
- `RecorderStartFailed(String)`
- `RecorderStopFailed(String)`
- `RecorderRequestDataFailed(String)`

---

### Camera

#### `use_camera() -> Camera`

Creates a camera controller.

`Camera` fields:

- `start: Callback<()>`
- `stop: Callback<()>`
- `stream: Signal<Option<web_sys::MediaStream>>`
- `state: Signal<CameraState>`
- `last_error: Signal<Option<String>>`

`Camera` methods:

- `is_active(&self) -> bool`  
  Returns `true` only while camera stream is active.
- `is_busy(&self) -> bool`  
  Returns `true` while transitioning (`Starting` or `Stopping`).

`CameraState`:

- `Idle`
- `Starting`
- `Active`
- `Stopping`
- `Error(String)`

`CameraError` variants:

- `WindowUnavailable`
- `MediaDevicesUnavailable`
- `GetUserMediaFailed(String)`
- `CastMediaStreamFailed`

---

### Storage

#### `use_storage<T>(key, default) -> Result<(Signal<T>, Callback<T>), StorageError>`
Where:

- `T: serde::Serialize + serde::de::DeserializeOwned + Clone + 'static`

Reads from `localStorage` and binds value to a `Signal<T>`.

- If key exists and decode succeeds, uses stored value
- Otherwise uses `default`
- Setter writes to `localStorage` and updates signal

#### `read_storage<T>(key) -> Result<Option<T>, StorageError>`
Reads one value from `localStorage` and deserializes it.

#### `write_storage<T>(key, value) -> Result<(), StorageError>`
Serializes and writes one value to `localStorage`.

`StorageError` variants:

- `WindowUnavailable`
- `LocalStorageUnavailable`
- `LocalStorageAccessFailed(String)`
- `ReadFailed(String)`
- `WriteFailed(String)`
- `SerializeFailed(String)`
- `DeserializeFailed(String)`

---

### Clipboard

#### `use_clipboard() -> Clipboard`

Creates clipboard helper.

`Clipboard` methods:

- `write(&self, text: &str) -> Callback<()>`  
  Returns a callback that writes text asynchronously.
- `write_async(&self, text: &str) -> Result<(), ClipboardError>`  
  Async write with explicit result.
- `read(&self) -> Result<Option<String>, ClipboardError>`  
  Async read with explicit result.

`ClipboardError` variants:

- `WindowUnavailable`
- `ClipboardUnavailable`
- `ReadFailed(String)`
- `WriteFailed(String)`

---

## Re-exports

```rust
pub use audio::{use_recording, Recording, RecordingError, RecordingState};
pub use camera::{use_camera, Camera, CameraError, CameraState};
pub use clipboard::{use_clipboard, Clipboard, ClipboardError};
pub use storage::{read_storage, use_storage, write_storage, StorageError};
```

---

## Usage Examples

### 1) Recording

```rust
use dioxus::prelude::*;
use pryty_rustbrowser::*;

#[component]
fn RecordingDemo() -> Element {
    let recording = use_recording();

    rsx! {
        button {
            onclick: move |_| recording.start.call(()),
            disabled: recording.is_active() || recording.is_busy(),
            "Start Recording"
        }
        button {
            onclick: move |_| recording.stop.call(()),
            disabled: !recording.is_active(),
            "Stop Recording"
        }
        p { "State: {recording.state.read():?}" }
        p {
            "Bytes: {
                recording
                    .data
                    .read()
                    .as_ref()
                    .map(|v| v.len().to_string())
                    .unwrap_or_else(|| \"0\".to_string())
            }"
        }
        {
            recording
                .last_error
                .read()
                .as_ref()
                .map(|e| rsx!(p { "Error: {e}" }))
        }
    }
}
```

### 2) Camera

```rust
use dioxus::prelude::*;
use pryty_rustbrowser::*;

#[component]
fn CameraDemo() -> Element {
    let camera = use_camera();

    rsx! {
        button {
            onclick: move |_| camera.start.call(()),
            disabled: camera.is_active() || camera.is_busy(),
            "Start Camera"
        }
        button {
            onclick: move |_| camera.stop.call(()),
            disabled: !camera.is_active(),
            "Stop Camera"
        }
        p { "State: {camera.state.read():?}" }
        {
            camera
                .last_error
                .read()
                .as_ref()
                .map(|e| rsx!(p { "Error: {e}" }))
        }
    }
}
```

### 3) Typed storage

```rust
use dioxus::prelude::*;
use pryty_rustbrowser::*;

#[component]
fn StorageDemo() -> Element {
    let storage = use_storage("username", String::new());

    let (name, set_name) = match storage {
        Ok(v) => v,
        Err(e) => {
            return rsx! { p { "Storage init failed: {e}" } };
        }
    };

    rsx! {
        input {
            value: "{name}",
            oninput: move |e| set_name.call(e.value())
        }
    }
}
```

### 4) Clipboard

```rust
use dioxus::prelude::*;
use pryty_rustbrowser::*;

#[component]
fn ClipboardDemo() -> Element {
    let clipboard = use_clipboard();
    let mut text = use_signal(String::new);

    let copy = {
        let value = text.read().clone();
        clipboard.write(&value)
    };

    rsx! {
        input {
            value: "{text}",
            oninput: move |e| text.set(e.value())
        }
        button { onclick: move |_| copy.call(()), "Copy" }
    }
}
```

---

## Notes

- This crate is designed for browser targets (`wasm32`).
- Browser permission policies apply to camera/microphone/clipboard APIs.
- Some features require secure context (HTTPS or localhost).

## License

Apache-2.0
