use dioxus::prelude::*;

pub fn use_storage(key: String) -> (Signal<String>, Callback<String>) {
    let storage = web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap();

    let initial = storage
        .get_item(&key)
        .ok()
        .flatten()
        .unwrap_or_default();

    let mut value = use_signal(|| initial);

    let setter = use_callback(move |new_val: String| {
        let _ = storage.set_item(&key, &new_val);
        value.set(new_val);
    });

    (value, setter)
}