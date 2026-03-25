use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaStream, MediaStreamConstraints};

pub struct Camera {
    pub start: Callback<()>,
    pub stop: Callback<()>,
    pub stream: Signal<Option<MediaStream>>,
    pub active: Signal<bool>,
}

pub fn use_camera() -> Camera {
    let mut stream = use_signal(|| None::<MediaStream>);
    let mut active = use_signal(|| false);

    let start = use_callback(move |_| {
        let mut stream = stream.clone();
        let mut active = active.clone();

        spawn(async move {
            let window = web_sys::window().unwrap();
            let devices = window.navigator().media_devices().unwrap();
            let constraints = MediaStreamConstraints::new();
            constraints.set_video(&true.into());

            let promise = devices.get_user_media_with_constraints(&constraints).unwrap();
            let js_val = JsFuture::from(promise).await.unwrap();
            let s: MediaStream = js_val.dyn_into().unwrap();

            stream.set(Some(s));
            active.set(true);
        });
    });

    let stop = use_callback(move |_| {
        if let Some(s) = stream.read().as_ref() {
            let tracks = s.get_tracks();
            for i in 0..tracks.length() {
                let track = tracks.get(i);
                let _ = js_sys::Reflect::get(&track, &"stop".into())
                    .ok()
                    .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
                    .map(|f| f.call0(&track));
            }
        }
        stream.set(None);
        active.set(false);
    });

    Camera {
        start,
        stop,
        stream,
        active,
    }
}