use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, MediaRecorder, MediaRecorderOptions, MediaStream, MediaStreamConstraints};
use js_sys::Uint8Array;

pub struct Recording {
    pub start: Callback<()>,
    pub stop: Callback<()>,
    pub data: Signal<Option<Vec<u8>>>,
    pub active: Signal<bool>,
}

pub fn use_recording() -> Recording {
    let data = use_signal(|| None::<Vec<u8>>);
    let active = use_signal(|| false);
    let recorder = use_signal(|| None::<MediaRecorder>);
    let stream = use_signal(|| None::<MediaStream>);
    let chunks = use_signal(|| Vec::<Vec<u8>>::new());

    let start = use_callback(move |_| {
        let mut active = active.clone();
        let mut recorder = recorder.clone();
        let mut stream = stream.clone();
        let chunks = chunks.clone();

        spawn(async move {
            let window = web_sys::window().unwrap();
            let devices = window.navigator().media_devices().unwrap();
            let constraints = MediaStreamConstraints::new();
            constraints.set_audio(&true.into());

            let promise = devices.get_user_media_with_constraints(&constraints).unwrap();
            let js_val = JsFuture::from(promise).await.unwrap();
            let s: MediaStream = js_val.dyn_into().unwrap();
            stream.set(Some(s.clone()));

            let options = MediaRecorderOptions::new();
            let rec = MediaRecorder::new_with_media_stream_and_media_recorder_options(&s, &options).unwrap();

            let chunks_cb = chunks.clone();
            let ondata = Closure::wrap(Box::new(move |e: web_sys::Event| {
                let blob = e.target().unwrap().dyn_into::<Blob>().unwrap();
                let mut chunks = chunks_cb.clone();
                spawn(async move {
                    let buf = JsFuture::from(blob.array_buffer()).await.unwrap();
                    let bytes = Uint8Array::new(&buf).to_vec();
                    chunks.write().push(bytes);
                });
            }) as Box<dyn FnMut(_)>);

            rec.set_ondataavailable(Some(ondata.as_ref().unchecked_ref()));
            ondata.forget();

            rec.start_with_time_slice(1000).unwrap();
            recorder.set(Some(rec));
            active.set(true);
        });
    });

    let stop = use_callback(move |_| {
        let mut data = data.clone();
        let mut active = active.clone();
        let mut chunks = chunks.clone();

        if let Some(r) = recorder.read().as_ref() {
            let _ = r.stop();
        }
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
        let all: Vec<u8> = chunks.read().iter().flatten().cloned().collect();
        data.set(Some(all));
        chunks.write().clear();
        active.set(false);
    });

    Recording { start, stop, data, active }
}