use dioxus::prelude::*;
use js_sys::Uint8Array;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    BlobEvent, MediaRecorder, MediaRecorderOptions, MediaStream, MediaStreamConstraints,
    MediaStreamTrack,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingState {
    Idle,
    Starting,
    Recording,
    Stopping,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum RecordingError {
    WindowUnavailable,
    MediaDevicesUnavailable,
    GetUserMediaFailed(String),
    CastMediaStreamFailed,
    RecorderCreateFailed(String),
    RecorderStartFailed(String),
    RecorderStopFailed(String),
    BlobReadFailed(String),
}

impl core::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RecordingError::WindowUnavailable => write!(f, "window unavailable"),
            RecordingError::MediaDevicesUnavailable => write!(f, "media devices unavailable"),
            RecordingError::GetUserMediaFailed(e) => write!(f, "getUserMedia failed: {e}"),
            RecordingError::CastMediaStreamFailed => write!(f, "failed to cast to MediaStream"),
            RecordingError::RecorderCreateFailed(e) => write!(f, "failed to create recorder: {e}"),
            RecordingError::RecorderStartFailed(e) => write!(f, "failed to start recorder: {e}"),
            RecordingError::RecorderStopFailed(e) => write!(f, "failed to stop recorder: {e}"),
            RecordingError::BlobReadFailed(e) => write!(f, "failed to read blob: {e}"),
        }
    }
}

pub struct Recording {
    pub start: Callback<()>,
    pub stop: Callback<()>,
    pub data: Signal<Option<Vec<u8>>>,
    pub state: Signal<RecordingState>,
    pub last_error: Signal<Option<String>>,
}

impl Recording {
    pub fn is_active(&self) -> bool {
        matches!(
            *self.state.read(),
            RecordingState::Starting | RecordingState::Recording | RecordingState::Stopping
        )
    }
}

pub fn use_recording() -> Recording {
    let data = use_signal(|| None::<Vec<u8>>);
    let state = use_signal(|| RecordingState::Idle);
    let last_error = use_signal(|| None::<String>);

    let recorder = use_signal(|| None::<MediaRecorder>);
    let stream = use_signal(|| None::<MediaStream>);
    let chunks = use_signal(|| Vec::<Vec<u8>>::new());

    let start = {
        let mut state = state.clone();
        let mut last_error = last_error.clone();
        let mut recorder = recorder.clone();
        let mut stream = stream.clone();
        let mut chunks = chunks.clone();

        use_callback(move |_| {
            let mut state = state.clone();
            let mut last_error = last_error.clone();
            let mut recorder = recorder.clone();
            let mut stream = stream.clone();
            let mut chunks = chunks.clone();

            spawn(async move {
                if let Err(e) = start_recording(
                    &mut state,
                    &mut last_error,
                    &mut recorder,
                    &mut stream,
                    &mut chunks,
                )
                .await
                {
                    let msg = e.to_string();
                    state.set(RecordingState::Error(msg.clone()));
                    last_error.set(Some(msg));
                }
            });
        })
    };

    let stop = {
        let mut data = data.clone();
        let mut state = state.clone();
        let mut last_error = last_error.clone();
        let mut recorder = recorder.clone();
        let mut stream = stream.clone();
        let mut chunks = chunks.clone();

        use_callback(move |_| {
            let mut data = data.clone();
            let mut state = state.clone();
            let mut last_error = last_error.clone();
            let mut recorder = recorder.clone();
            let mut stream = stream.clone();
            let mut chunks = chunks.clone();

            if let Err(e) = stop_recording(
                &mut data,
                &mut state,
                &mut last_error,
                &mut recorder,
                &mut stream,
                &mut chunks,
            ) {
                let msg = e.to_string();
                state.set(RecordingState::Error(msg.clone()));
                last_error.set(Some(msg));
            }
        })
    };

    Recording {
        start,
        stop,
        data,
        state,
        last_error,
    }
}

async fn start_recording(
    state: &mut Signal<RecordingState>,
    last_error: &mut Signal<Option<String>>,
    recorder: &mut Signal<Option<MediaRecorder>>,
    stream: &mut Signal<Option<MediaStream>>,
    chunks: &mut Signal<Vec<Vec<u8>>>,
) -> Result<(), RecordingError> {
    state.set(RecordingState::Starting);
    last_error.set(None);
    chunks.write().clear();

    let window = web_sys::window().ok_or(RecordingError::WindowUnavailable)?;
    let devices = window
        .navigator()
        .media_devices()
        .map_err(|_| RecordingError::MediaDevicesUnavailable)?;

    let mut constraints = MediaStreamConstraints::new();
    constraints.set_audio(&true.into());

    let promise = devices
        .get_user_media_with_constraints(&constraints)
        .map_err(|e| RecordingError::GetUserMediaFailed(format!("{e:?}")))?;
    let js_val = JsFuture::from(promise)
        .await
        .map_err(|e| RecordingError::GetUserMediaFailed(format!("{e:?}")))?;
    let s: MediaStream = js_val
        .dyn_into()
        .map_err(|_| RecordingError::CastMediaStreamFailed)?;
    stream.set(Some(s.clone()));

    let options = MediaRecorderOptions::new();
    let rec = MediaRecorder::new_with_media_stream_and_media_recorder_options(&s, &options)
        .map_err(|e| RecordingError::RecorderCreateFailed(format!("{e:?}")))?;

    let chunks_for_data = chunks.clone();
    let ondata = Closure::wrap(Box::new(move |e: BlobEvent| {
        let blob = e.data();
        let mut chunks_inner = chunks_for_data.clone();

        spawn(async move {
            if let Ok(buf) = JsFuture::from(blob.array_buffer()).await {
                let bytes = Uint8Array::new(&buf).to_vec();
                chunks_inner.write().push(bytes);
            }
        });
    }) as Box<dyn FnMut(_)>);

    rec.set_ondataavailable(Some(ondata.as_ref().unchecked_ref()));
    ondata.forget();

    rec.start_with_time_slice(1000)
        .map_err(|e| RecordingError::RecorderStartFailed(format!("{e:?}")))?;
    recorder.set(Some(rec));
    state.set(RecordingState::Recording);

    Ok(())
}

fn stop_recording(
    data: &mut Signal<Option<Vec<u8>>>,
    state: &mut Signal<RecordingState>,
    _last_error: &mut Signal<Option<String>>,
    recorder: &mut Signal<Option<MediaRecorder>>,
    stream: &mut Signal<Option<MediaStream>>,
    chunks: &mut Signal<Vec<Vec<u8>>>,
) -> Result<(), RecordingError> {
    state.set(RecordingState::Stopping);

    if let Some(r) = recorder.read().as_ref() {
        r.stop()
            .map_err(|e| RecordingError::RecorderStopFailed(format!("{e:?}")))?;
    }

    if let Some(s) = stream.read().as_ref() {
        let tracks = s.get_tracks();
        for i in 0..tracks.length() {
            if let Ok(track) = tracks.get(i).dyn_into::<MediaStreamTrack>() {
                track.stop();
            }
        }
    }

    let all: Vec<u8> = chunks.read().iter().flatten().cloned().collect();
    data.set(Some(all));
    chunks.write().clear();

    recorder.set(None);
    stream*
