use dioxus::prelude::*;
use js_sys::{Promise, Uint8Array};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    BlobEvent, MediaRecorder, MediaRecorderOptions, MediaStream, MediaStreamConstraints,
    MediaStreamTrack,
};
/*
stream = the audio data captured during recording
chunks = stored stream that is organized
*/

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingState {
    Idle,
    Starting,
    Recording,
    Paused,
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
    RecorderRequestDataFailed(String),
    RecorderPauseFailed(String),
    RecorderResumeFailed(String),
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
            RecordingError::RecorderRequestDataFailed(e) => {
                write!(f, "failed to request recorder data: {e}")
            }
            RecordingError::RecorderPauseFailed(e) => write!(f, "failed to pause recorder: {e}"),
            RecordingError::RecorderResumeFailed(e) => write!(f, "failed to resume recorder: {e}"),
        }
    }
}

pub struct AudioQualityConfig {
    pub name: &'static str,
    pub sample_rate: f64,
    pub sample_size: u32,
    pub channel_count: u32,
    pub bits_per_second: u32,
    pub mime_type: &'static str,
}

impl AudioQualityConfig {
    pub fn low() -> Self {
        Self {
            name: "Low quality (voice call)",
            sample_rate: 22050.0,
            sample_size: 16,
            channel_count: 1,
            bits_per_second: 64000,
            mime_type: "audio/webm;codecs=opus",
        }
    }

    pub fn normal() -> Self {
        Self {
            name: "Standard quality",
            sample_rate: 44100.0,
            sample_size: 16,
            channel_count: 1,
            bits_per_second: 128000,
            mime_type: "audio/webm;codecs=opus",
        }
    }

    pub fn high() -> Self {
        Self {
            name: "High quality",
            sample_rate: 48000.0,
            sample_size: 24,
            channel_count: 2,
            bits_per_second: 192000,
            mime_type: "audio/webm;codecs=opus",
        }
    }

    pub fn studio() -> Self {
        Self {
            name: "Studio quality",
            sample_rate: 96000.0,
            sample_size: 24,
            channel_count: 2,
            bits_per_second: 320000,
            mime_type: "audio/webm;codecs=opus",
        }
    }

    pub fn lossless() -> Self {
        Self {
            name: "Lossless quality",
            sample_rate: 96000.0,
            sample_size: 32,
            channel_count: 2,
            bits_per_second: 0,
            mime_type: "audio/webm;codecs=pcm",
        }
    }
}

// 建议增加的功能
#[derive(Debug, Clone, Copy, Default)]
pub struct RecordingConfig {
    pub time_slice: Option<u32>,  // 可配置数据块间隔
    pub max_duration: Option<u32>, // 最大录音时长
}

pub struct Recording {
    pub start: Callback<()>,
    pub start_with_quality: Callback<AudioQualityConfig>,
    pub start_with_config: Callback<RecordingConfig>,
    pub start_with_quality_and_config: Callback<(AudioQualityConfig, RecordingConfig)>,
    pub pause: Callback<()>,
    pub resume: Callback<()>,
    pub stop: Callback<()>,
    pub data: Signal<Option<Vec<u8>>>,
    pub state: Signal<RecordingState>,
    pub last_error: Signal<Option<String>>,
    recorder: Signal<Option<MediaRecorder>>,
    stream: Signal<Option<MediaStream>>,
    chunks: Signal<Vec<Vec<u8>>>,
}

impl Recording {
    pub fn is_active(&self) -> bool {
        matches!(*self.state.read(), RecordingState::Recording)
    }

    pub fn is_paused(&self) -> bool {
        matches!(*self.state.read(), RecordingState::Paused)
    }

    pub fn is_busy(&self) -> bool {
        matches!(
            *self.state.read(),
            RecordingState::Starting | RecordingState::Stopping
        )
    }
}

// 更好的资源清理
impl Drop for Recording {
    fn drop(&mut self) {
        // 增加手动drop减少内存问题
        let _ = stop_recording(
            &mut self.data,
            &mut self.state,
            &mut self.last_error,
            &mut self.recorder,
            &mut self.stream,
            &mut self.chunks,
        );
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
        let state = state.clone();
        let last_error = last_error.clone();
        let recorder = recorder.clone();
        let stream = stream.clone();
        let chunks = chunks.clone();

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
                    // if error then change error msg
                    let msg = e.to_string();
                    state.set(RecordingState::Error(msg.clone()));
                    last_error.set(Some(msg));
                }
            });
        })
    };

    let start_with_quality = {
        let state = state.clone();
        let last_error = last_error.clone();
        let recorder = recorder.clone();
        let stream = stream.clone();
        let chunks = chunks.clone();

        use_callback(move |quality: AudioQualityConfig| {
            let mut state = state.clone();
            let mut last_error = last_error.clone();
            let mut recorder = recorder.clone();
            let mut stream = stream.clone();
            let mut chunks = chunks.clone();

            spawn(async move {
                if let Err(e) = start_rec_with_quality_and_config(
                    &mut state,
                    &mut last_error,
                    &mut recorder,
                    &mut stream,
                    &mut chunks,
                    Some(quality),
                    None,
                )
                .await
                {
                    // if error then change error msg
                    let msg = e.to_string();
                    state.set(RecordingState::Error(msg.clone()));
                    last_error.set(Some(msg));
                }
            });
        })
    };

    let start_with_config = {
        let state = state.clone();
        let last_error = last_error.clone();
        let recorder = recorder.clone();
        let stream = stream.clone();
        let chunks = chunks.clone();

        use_callback(move |config: RecordingConfig| {
            let mut state = state.clone();
            let mut last_error = last_error.clone();
            let mut recorder = recorder.clone();
            let mut stream = stream.clone();
            let mut chunks = chunks.clone();

            spawn(async move {
                if let Err(e) = start_rec_with_quality_and_config(
                    &mut state,
                    &mut last_error,
                    &mut recorder,
                    &mut stream,
                    &mut chunks,
                    None,
                    Some(config),
                )
                .await
                {
                    // if error then change error msg
                    let msg = e.to_string();
                    state.set(RecordingState::Error(msg.clone()));
                    last_error.set(Some(msg));
                }
            });
        })
    };

    let start_with_quality_and_config = {
        let state = state.clone();
        let last_error = last_error.clone();
        let recorder = recorder.clone();
        let stream = stream.clone();
        let chunks = chunks.clone();

        use_callback(move |(quality, config): (AudioQualityConfig, RecordingConfig)| {
            let mut state = state.clone();
            let mut last_error = last_error.clone();
            let mut recorder = recorder.clone();
            let mut stream = stream.clone();
            let mut chunks = chunks.clone();

            spawn(async move {
                if let Err(e) = start_rec_with_quality_and_config(
                    &mut state,
                    &mut last_error,
                    &mut recorder,
                    &mut stream,
                    &mut chunks,
                    Some(quality),
                    Some(config),
                )
                .await
                {
                    // if error then change error msg
                    let msg = e.to_string();
                    state.set(RecordingState::Error(msg.clone()));
                    last_error.set(Some(msg));
                }
            });
        })
    };

    let pause = {
        let state = state.clone();
        let last_error = last_error.clone();
        let recorder = recorder.clone();

        use_callback(move |_| {
            let mut state = state.clone();
            let mut last_error = last_error.clone();
            let mut recorder = recorder.clone();

            if let Err(e) = pause_recording(&mut state, &mut last_error, &mut recorder) {
                let msg = e.to_string();
                state.set(RecordingState::Error(msg.clone()));
                last_error.set(Some(msg));
            }
        })
    };

    let resume = {
        let state = state.clone();
        let last_error = last_error.clone();
        let recorder = recorder.clone();

        use_callback(move |_| {
            let mut state = state.clone();
            let mut last_error = last_error.clone();
            let mut recorder = recorder.clone();

            if let Err(e) = resume_recording(&mut state, &mut last_error, &mut recorder) {
                let msg = e.to_string();
                state.set(RecordingState::Error(msg.clone()));
                last_error.set(Some(msg));
            }
        })
    };

    let stop = {
        let data = data.clone();
        let state = state.clone();
        let last_error = last_error.clone();
        let recorder = recorder.clone();
        let stream = stream.clone();
        let chunks = chunks.clone();

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
        start_with_quality,
        start_with_config,
        start_with_quality_and_config,
        pause,
        resume,
        stop,
        data,
        state,
        last_error,
        recorder,
        stream,
        chunks,
    }
}

async fn start_recording(
    state: &mut Signal<RecordingState>,
    last_error: &mut Signal<Option<String>>,
    recorder: &mut Signal<Option<MediaRecorder>>,
    stream: &mut Signal<Option<MediaStream>>,
    chunks: &mut Signal<Vec<Vec<u8>>>,
) -> Result<(), RecordingError> {
    start_rec_with_quality_and_config(state, last_error, recorder, stream, chunks, None, None).await
}

pub async fn start_rec_with_quality(
    state: &mut Signal<RecordingState>,
    last_error: &mut Signal<Option<String>>,
    recorder: &mut Signal<Option<MediaRecorder>>,
    stream: &mut Signal<Option<MediaStream>>,
    chunks: &mut Signal<Vec<Vec<u8>>>,
    quality: Option<AudioQualityConfig>,
) -> Result<(), RecordingError> {
    start_rec_with_quality_and_config(state, last_error, recorder, stream, chunks, quality, None).await
}

pub async fn start_rec_with_quality_and_config(
    state: &mut Signal<RecordingState>,
    last_error: &mut Signal<Option<String>>,
    recorder: &mut Signal<Option<MediaRecorder>>,
    stream: &mut Signal<Option<MediaStream>>,
    chunks: &mut Signal<Vec<Vec<u8>>>,
    quality: Option<AudioQualityConfig>,
    config: Option<RecordingConfig>,
) -> Result<(), RecordingError> {
    if matches!(
        *state.read(),
        RecordingState::Starting | RecordingState::Recording | RecordingState::Paused
    ) {
        return Ok(());
    }

    state.set(RecordingState::Starting);
    if let Some(s) = stream.read().as_ref() {
        let tracks = s.get_tracks();
        for i in 0..tracks.length() {
            if let Ok(track) = tracks.get(i).dyn_into::<MediaStreamTrack>() {
                track.stop();
            }
        }
    }
    last_error.set(None);
    chunks.write().clear();
    stream.set(None);
    let window = web_sys::window().ok_or(RecordingError::WindowUnavailable)?;
    let devices = window
        .navigator()
        .media_devices()
        .map_err(|_| RecordingError::MediaDevicesUnavailable)?;

    let constraints = MediaStreamConstraints::new();
    constraints.set_audio(&true.into());

    let promise: Promise = devices
        .get_user_media_with_constraints(&constraints)
        .map_err(|e| RecordingError::GetUserMediaFailed(format!("{e:?}")))?;

    let js_val: JsValue = JsFuture::from(promise)
        .await
        .map_err(|e| RecordingError::GetUserMediaFailed(format!("{e:?}")))?;

    let s: MediaStream = js_val
        .dyn_into()
        .map_err(|_| RecordingError::CastMediaStreamFailed)?;

    stream.set(Some(s.clone()));

    let options = MediaRecorderOptions::new();
    if let Some(q) = quality {
        options.set_mime_type(q.mime_type);
        if q.bits_per_second > 0 {
            options.set_audio_bits_per_second(q.bits_per_second);
        }
    }

    let rec = MediaRecorder::new_with_media_stream_and_media_recorder_options(&s, &options)
        .map_err(|e| RecordingError::RecorderCreateFailed(format!("{e:?}")))?;

    let chunks_for_data = chunks.clone();
    let ondata = Closure::wrap(Box::new(move |e: BlobEvent| {
        let maybe_blob = e.data();
        let mut chunks_inner = chunks_for_data.clone();

        spawn(async move {
            if let Some(blob) = maybe_blob {
                if let Ok(buf) = JsFuture::from(blob.array_buffer()).await {
                    let bytes = Uint8Array::new(&buf).to_vec();
                    chunks_inner.write().push(bytes);
                }
            }
        });
    }) as Box<dyn FnMut(_)>);

    rec.set_ondataavailable(Some(ondata.as_ref().unchecked_ref()));
    ondata.forget(); /*If we forget to free it, Rust won't free the memory — it will just let the browser free it.
     Otherwise, it could lead to double free or similar issues.(Bad things!☆*: .｡. o(≧▽≦)o .｡.:*☆)
 */

    let time_slice = config.and_then(|c| c.time_slice).unwrap_or(1000);
    rec.start_with_time_slice(time_slice)
        .map_err(|e| RecordingError::RecorderStartFailed(format!("{e:?}")))?;

    recorder.set(Some(rec));
    state.set(RecordingState::Recording);

    if let Some(max_duration) = config.and_then(|c| c.max_duration) {
        let mut data_for_timeout = Signal::new(None::<Vec<u8>>);
        let mut state_for_timeout = state.clone();
        let mut last_error_for_timeout = last_error.clone();
        let mut recorder_for_timeout = recorder.clone();
        let mut stream_for_timeout = stream.clone();
        let mut chunks_for_timeout = chunks.clone();

        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(max_duration).await;
            let _ = stop_recording(
                &mut data_for_timeout,
                &mut state_for_timeout,
                &mut last_error_for_timeout,
                &mut recorder_for_timeout,
                &mut stream_for_timeout,
                &mut chunks_for_timeout,
            );
        });
    }

    Ok(())
}

fn pause_recording(
    state: &mut Signal<RecordingState>,
    _last_error: &mut Signal<Option<String>>,
    recorder: &mut Signal<Option<MediaRecorder>>,
) -> Result<(), RecordingError> {
    if !matches!(*state.read(), RecordingState::Recording) {
        return Ok(());
    }

    if let Some(r) = recorder.read().as_ref() {
        r.request_data()
            .map_err(|e| RecordingError::RecorderRequestDataFailed(format!("{e:?}")))?;
        r.pause()
            .map_err(|e| RecordingError::RecorderPauseFailed(format!("{e:?}")))?;
        state.set(RecordingState::Paused);
    }

    Ok(())
}

fn resume_recording(
    state: &mut Signal<RecordingState>,
    _last_error: &mut Signal<Option<String>>,
    recorder: &mut Signal<Option<MediaRecorder>>,
) -> Result<(), RecordingError> {
    if !matches!(*state.read(), RecordingState::Paused) {
        return Ok(());
    }

    if let Some(r) = recorder.read().as_ref() {
        r.resume()
            .map_err(|e| RecordingError::RecorderResumeFailed(format!("{e:?}")))?;
        state.set(RecordingState::Recording);
    }

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
    if matches!(*state.read(), RecordingState::Idle) {
        return Ok(());
    }

    state.set(RecordingState::Stopping);

    if let Some(r) = recorder.read().as_ref() {
        r.request_data()
            .map_err(|e| RecordingError::RecorderRequestDataFailed(format!("{e:?}")))?;
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
    stream.set(None);
    state.set(RecordingState::Idle);

    Ok(())
}
