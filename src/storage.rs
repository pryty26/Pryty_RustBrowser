use dioxus::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum StorageError {
    WindowUnavailable,
    LocalStorageUnavailable,
    LocalStorageAccessFailed(String),
    ReadFailed(String),
    WriteFailed(String),
    SerializeFailed(String),
    DeserializeFailed(String),
}

impl core::fmt::Display for StorageError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StorageError::WindowUnavailable => write!(f, "window unavailable"),
            StorageError::LocalStorageUnavailable => write!(f, "localStorage unavailable"),
            StorageError::LocalStorageAccessFailed(e) => {
                write!(f, "failed to access localStorage: {e}")
            }
            StorageError::ReadFailed(e) => write!(f, "failed to read from localStorage: {e}"),
            StorageError::WriteFailed(e) => write!(f, "failed to write to localStorage: {e}"),
            StorageError::SerializeFailed(e) => write!(f, "failed to serialize value: {e}"),
            StorageError::DeserializeFailed(e) => write!(f, "failed to deserialize value: {e}"),
        }
    }
}

fn get_local_storage() -> Result<web_sys::Storage, StorageError> {
    let window = web_sys::window().ok_or(StorageError::WindowUnavailable)?;
    let storage = window
        .local_storage()
        .map_err(|e| StorageError::LocalStorageAccessFailed(format!("{e:?}")))?
        .ok_or(StorageError::LocalStorageUnavailable)?;
    Ok(storage)
}

pub fn use_storage<T>(
    key: impl Into<String>,
    default: T,
) -> Result<(Signal<T>, Callback<T>), StorageError>
where
    T: Serialize + DeserializeOwned + Clone + 'static,
{
    let key = key.into();
    let storage = get_local_storage()?;

    let initial = match storage.get_item(&key) {
        Ok(Some(raw)) => match serde_json::from_str::<T>(&raw) {
            Ok(v) => v,
            Err(_) => default.clone(),
        },
        Ok(None) => default.clone(),
        Err(_) => default.clone(),
    };

    let mut value = use_signal(|| initial);

    let setter = {
        let key = key.clone();
        let storage = storage.clone();

        use_callback(move |new_val: T| {
            if let Ok(serialized) = serde_json::to_string(&new_val) {
                let _ = storage
                    .set_item(&key, &serialized)
                    .map_err(|e| StorageError::WriteFailed(format!("{e:?}")));
            }
            value.set(new_val);
        })
    };

    Ok((value, setter))
}

pub fn read_storage<T>(key: impl AsRef<str>) -> Result<Option<T>, StorageError>
where
    T: DeserializeOwned,
{
    let storage = get_local_storage()?;
    let key_ref = key.as_ref();

    let raw = storage
        .get_item(key_ref)
        .map_err(|e| StorageError::ReadFailed(format!("{e:?}")))?;

    match raw {
        Some(text) => {
            let parsed = serde_json::from_str::<T>(&text)
                .map_err(|e| StorageError::DeserializeFailed(e.to_string()))?;
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

pub fn write_storage<T>(key: impl AsRef<str>, value: &T) -> Result<(), StorageError>
where
    T: Serialize,
{
    let storage = get_local_storage()?;
    let key_ref = key.as_ref();

    let serialized =
        serde_json::to_string(value).map_err(|e| StorageError::SerializeFailed(e.to_string()))?;

    storage
        .set_item(key_ref, &serialized)
        .map_err(|e| StorageError::WriteFailed(format!("{e:?}")))?;

    Ok(())
}