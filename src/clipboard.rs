use dioxus::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardError {
    WindowUnavailable,
    ClipboardUnavailable,
    ReadFailed(String),
    WriteFailed(String),
}

impl core::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ClipboardError::WindowUnavailable => write!(f, "window unavailable"),
            ClipboardError::ClipboardUnavailable => write!(f, "clipboard unavailable"),
            ClipboardError::ReadFailed(e) => write!(f, "clipboard read failed: {e}"),
            ClipboardError::WriteFailed(e) => write!(f, "clipboard write failed: {e}"),
        }
    }
}

pub struct Clipboard;

pub fn use_clipboard() -> Clipboard {
    Clipboard
}

impl Clipboard {
    pub fn write(&self, text: &str) -> Callback<()> {
        let payload = text.to_owned();
        Callback::new(move |_| {
            let payload = payload.clone();
            spawn(async move {
                let _ = write_clipboard_text(payload).await;
            });
        })
    }

    pub async fn write_async(&self, text: &str) -> Result<(), ClipboardError> {
        write_clipboard_text(text.to_owned()).await
    }

    pub async fn read(&self) -> Result<Option<String>, ClipboardError> {
        let window = web_sys::window().ok_or(ClipboardError::WindowUnavailable)?;
        let clipboard = window.navigator().clipboard();

        let value = JsFuture::from(clipboard.read_text())
            .await
            .map_err(|e| ClipboardError::ReadFailed(format!("{e:?}")))?;

        Ok(value.as_string())
    }
}

async fn write_clipboard_text(text: String) -> Result<(), ClipboardError> {
    let window = web_sys::window().ok_or(ClipboardError::WindowUnavailable)?;
    let clipboard = window.navigator().clipboard();

    JsFuture::from(clipboard.write_text(&text))
        .await
        .map_err(|e| ClipboardError::WriteFailed(format!("{e:?}")))?;

    Ok(())
}