use dioxus::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub fn use_clipboard() -> Clipboard {
    Clipboard
}

pub struct Clipboard;

impl Clipboard {
    pub fn write(&self, text: &str) {
        let text = text.to_owned();
        spawn(async move {
            let window = web_sys::window().unwrap();
            let _ = JsFuture::from(window.navigator().clipboard().write_text(&text)).await;
        });
    }

    pub async fn read(&self) -> Option<String> {
        let window = web_sys::window().unwrap();
        JsFuture::from(window.navigator().clipboard().read_text())
            .await
            .ok()
            .and_then(|v| v.as_string())
    }
}