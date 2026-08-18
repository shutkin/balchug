use dioxus::prelude::*;
use web_sys::window;

pub const KEY_PROJECT_ID: &str = "project_id";

pub struct LocalStorage {}

impl LocalStorage {
    fn get_storage() -> Option<web_sys::Storage> {
        window()
            .and_then(|window| {
                window.local_storage().unwrap_or_else(|err| {
                    error!("Failed to get local storage: {err:?}");
                    None
                })
            })
    }

    pub fn get(key: &str) -> Option<String> {
        Self::get_storage()
            .and_then(|storage| {
                storage.get_item(key).unwrap_or_else(|err| {
                    error!("Failed to get storage object: {err:?}");
                    None
                })
            })
    }

    pub fn set(key: &str, value: &str) {
        info!("Storage: {key}={value}");
        if let Some(storage) = Self::get_storage() {
            storage.set_item(key, value).unwrap_or_else(|err| {
                error!("Failed to set storage object: {err:?}");
            });
        }
    }

    pub fn remove(key: &str) {
        info!("Storage: remove {key}");
        if let Some(storage) = Self::get_storage() {
            storage.remove_item(key).unwrap_or_else(|err| {
                error!("Failed to remove storage object: {err:?}");
            });
        }
    }
}
