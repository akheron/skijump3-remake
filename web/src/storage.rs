use crate::warn;
use common::files::{MemoryStore, Store, WritableStore};
use std::borrow::Cow;
use wasm_bindgen::JsValue;
use web_sys::Storage;

/// The files that the game writes. They stay in localStorage after the page
/// closes. The game reads them from a copy in memory, so that it can go on
/// with the files in memory when localStorage cannot be used.
pub struct LocalStorageStore {
    memory: MemoryStore,
    storage: Option<Storage>,
}

/// localStorage is shared by all pages of the same origin
const KEY_PREFIX: &str = "skijump3/";

impl LocalStorageStore {
    pub fn new() -> Self {
        let memory = MemoryStore::default();
        let storage = match local_storage() {
            Ok(storage) => Some(storage),
            Err(err) => {
                warn(&format!(
                    "localStorage cannot be used, so the files are not saved: {err:?}"
                ));
                None
            }
        };
        if let Some(storage) = &storage {
            if let Err(err) = load(storage, &memory) {
                warn(&format!(
                    "Unable to read the files from localStorage: {err:?}"
                ));
            }
        }
        Self { memory, storage }
    }

    /// Tests do not run in a browser
    #[cfg(test)]
    pub fn in_memory() -> Self {
        Self {
            memory: MemoryStore::default(),
            storage: None,
        }
    }
}

impl Store for LocalStorageStore {
    fn read(&self, name: &str) -> Option<Cow<'_, [u8]>> {
        self.memory.read(name)
    }
    fn names(&self) -> Vec<String> {
        self.memory.names()
    }
}

impl WritableStore for LocalStorageStore {
    fn write(&self, name: &str, content: &[u8]) {
        self.memory.write(name, content);
        if let Some(storage) = &self.storage {
            let key = format!("{KEY_PREFIX}{name}");
            if let Err(err) = storage.set_item(&key, &encode(content)) {
                warn(&format!("Unable to save {name} to localStorage: {err:?}"));
            }
        }
    }
    fn remove(&self, name: &str) {
        self.memory.remove(name);
        if let Some(storage) = &self.storage {
            let key = format!("{KEY_PREFIX}{name}");
            if let Err(err) = storage.remove_item(&key) {
                warn(&format!(
                    "Unable to remove {name} from localStorage: {err:?}"
                ));
            }
        }
    }
}

fn local_storage() -> Result<Storage, JsValue> {
    web_sys::window()
        .ok_or_else(|| JsValue::from_str("no window"))?
        .local_storage()?
        .ok_or_else(|| JsValue::from_str("no localStorage"))
}

fn load(storage: &Storage, memory: &MemoryStore) -> Result<(), JsValue> {
    for index in 0..storage.length()? {
        let Some(key) = storage.key(index)? else {
            continue;
        };
        let Some(name) = key.strip_prefix(KEY_PREFIX) else {
            continue;
        };
        let Some(value) = storage.get_item(&key)? else {
            continue;
        };
        match decode(&value) {
            Some(content) => memory.write(name, &content),
            None => warn(&format!("Ignoring {key} in localStorage: not a game file")),
        }
    }
    Ok(())
}

/// localStorage keeps strings. Each byte is one character from U+0000 to
/// U+00FF, so the string keeps all bytes and takes one UTF-16 unit per byte.
fn encode(content: &[u8]) -> String {
    content.iter().map(|&byte| byte as char).collect()
}

fn decode(value: &str) -> Option<Vec<u8>> {
    value.chars().map(|c| u8::try_from(c).ok()).collect()
}
