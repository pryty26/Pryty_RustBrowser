use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MediaStream, MediaStreamConstraints, MediaStreamTrack};

#[derive(Debug, Clone, PartialEq)]
pub enum CameraState {
    Idle,
    Starting,
    Active,
    Stopping,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum CameraError {
    WindowUnavailable,
    MediaDevicesUnavailable,
    GetUserMediaFailed(String),
    CastMediaStreamFailed,
}

impl core::fmt::Display for CameraError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CameraError::WindowUnavailable => write!(f, "window unavailable"),
            CameraError::MediaDevicesUnavailable => write!(f, "media devices unavailable"),
            CameraError::GetUserMediaFailed(e) => write!(f, "getUserMedia failed: {e}"),
            CameraError::CastMediaStreamFailed => write!(f, "failed to cast to MediaStream"),
        }
    }
}

pub struct Camera {
    pub start: Callback<()>,
    pub stop: Callback<()>,
    pub stream: Signal<Option<MediaStream>>,
    pub state: Signal<CameraState>,
    pub last_error: Signal<Option<String>>,
}

impl Camera {
    pub fn is_active(&self) -> bool {
        matches!(*self.state.read(), CameraState::Active)
    }

    pub fn is_busy(&self) -> bool {
        matches!(*self.state.read(), CameraState::Starting | CameraState::Stopping)
    }
}

pub fn use_camera() -> Camera {
    let stream = use_signal(|| None::<MediaStream>);
    let state = use_signal(|| CameraState::Idle);
    let last_error = use_signal(|| None::<String>);

    let start = {
        let mut stream = stream.clone();
        let mut state = state.clone();
        let mut last_error = last_error.clone();

        use_callback(move |_| {
            let mut stream = stream.clone();
            let mut state = state.clone();
            let mut last_error = last_error.clone();

            spawn(async move {
                if let Err(e) = start_camera(&mut stream, &mut state, &mut last_error).await {
                    let msg = e.to_string();
                    state.set(CameraState::Error(msg.clone()));
                    last_error.set(Some(msg));
                }
            });
        })
    };

    let stop = {
        let mut stream = stream.clone();
        let mut state = state.clone();

        use_callback(move |_| {
            stop_camera(&mut stream, &mut state);
        })
    };

    Camera {
        start,
        stop,
        stream,
        state,
        last_error,
    }
}

async fn start_camera(
    stream: &mut Signal<Option<MediaStream>>,
    state: &mut Signal<CameraState>,
    last_error: &mut Signal<Option<String>>,
) -> Result<(), CameraError> {
    state.set(CameraState::Starting);
    last_error.set(None);

    let window = web_sys::window().ok_or(CameraError::WindowUnavailable)?;
    let devices = window
        .navigator()
        .media_devices()
        .map_err(|_| CameraError::MediaDevicesUnavailable)?;

    let constraints = MediaStreamConstraints::new();
    constraints.set_video(&true.into());

    let promise = devices
        .get_user_media_with_constraints(&constraints)
        .map_err(|e| CameraError::GetUserMediaFailed(format!("{e:?}")))?;

    let js_val = JsFuture::from(promise)
        .await
        .map_err(|e| CameraError::GetUserMediaFailed(format!("{e:?}")))?;

    let s: MediaStream = js_val
        .dyn_into()
        .map_err(|_| CameraError::CastMediaStreamFailed)?;

    stream.set(Some(s));
    state.set(CameraState::Active);
    Ok(())
}

fn stop_camera(stream: &mut Signal<Option<MediaStream>>, state: &mut Signal<CameraState>) {
    state.set(CameraState::Stopping);

    if let Some(s) = stream.read().as_ref() {
        let tracks = s.get_tracks();
        for i in 0..tracks.length() {
            if let Ok(track) = tracks.get(i).dyn_into::<MediaStreamTrack>() {
                track.stop();
            }
        }
    }

    stream.set(None);
    state.set(CameraState::Idle);
}