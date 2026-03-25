Pryty_RustBrowser

A collection of browser API hooks for Rust developer.
Install

cargo add pryty-rustbrowser

Hooks

use_recording() - start/stop recording, get audio bytes

use_camera() - start/stop camera, get video stream

use_storage(key) - read/write localStorage

use_clipboard() - copy/paste text

Example

use pryty_rustbrowser::*;

fn App() -> Element { let rec = use_recording(); let (name, set_name) = use_storage("username".to_string());

rsx! {
    button { onclick: move |_| rec.start(), "Record" }
    input { value: "{name}", oninput: move |e| set_name(e.value()) }
}

}

License

Apache-2.0
