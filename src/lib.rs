// Example
//
// ```
// use pryty_rustbrowser::*;
//
// fn App() -> Element {
//     let rec = use_recording();
//     let (name, set_name) = use_storage("username".to_string());
//
//     rsx! {
//         button { onclick: move |_| rec.start(), "录音" }
//         input { value: "{name}", oninput: move |e| set_name(e.value()) }
//     }
// }
// ```
// src/lib.rs
mod audio;
mod camera;
mod clipboard;
mod storage;

pub use audio::{
    use_recording,
    AudioQualityConfig,
    Recording,
    RecordingError,
    RecordingState,
};
pub use camera::{
    use_camera,
    Camera,
    CameraError,
    CameraQualityConfig,
    CameraState,
};
pub use clipboard::{use_clipboard, Clipboard, ClipboardError};
pub use storage::{read_storage, use_storage, write_storage, StorageError};
/*
use dioxus::prelude::*;
use pryty_rustbrowser::{use_camera, use_clipboard, use_recording, use_storage};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let camera = use_camera();
    let recording = use_recording();
    let (name, set_name) = use_storage("username".to_string());
    let clipboard = use_clipboard();

    let on_copy = {
        let name = name.clone();
        move |_| {
            clipboard.write(&name.read());
        }
    };

    rsx! {
        div {
            h1 { "pryty-rustbrowser demo" }

            div {
                input {
                    value: "{name}",
                    oninput: move |e| set_name.call(e.value())
                }
                button { onclick: on_copy, "Copy Username" }
            }

            div {
                button {
                    onclick: move |_| camera.start.call(()),
                    disabled: *camera.active.read(),
                    "Start Camera"
                    
                }
                button {
                    onclick: move |_| camera.stop.call(()),
                    disabled: !*camera.active.read(),
                    "Stop Camera"
                }
                p { "camera active: {camera.active.read()}" }
            }

            div {
                button {
                    onclick: move |_| recording.start.call(()),
                    disabled: *recording.active.read(),
                    "Start Recording"
                }
                button {
                    onclick: move |_| recording.start_with_quality.call(AudioQualityConfig::high()),
                    "Start Recording"
                }
                button {
                    onclick: move |_| recording.stop.call(()),
                    disabled: !*recording.active.read(),
                    "Stop Recording"
                }
                p { "recording active: {recording.active.read()}" }
                p {
                    "bytes: {
                        recording
                            .data
                            .read()
                            .as_ref()
                            .map(|v| v.len().to_string())
                            .unwrap_or_else(|| \"0\".to_string())
                    }"
                }
            }
        }
    }
} */