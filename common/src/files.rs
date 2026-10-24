//! The data files of the game. The files that the game writes go to a
//! writable store, and the game reads them back from there. The other files
//! come from the read-only assets.

use crate::trace;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Write;

/// A flat directory of files
pub trait Store {
    fn read(&self, name: &str) -> Option<Cow<'_, [u8]>>;
    fn contains(&self, name: &str) -> bool {
        self.read(name).is_some()
    }
    fn names(&self) -> Vec<String>;
}

pub trait WritableStore: Store {
    fn write(&self, name: &str, content: &[u8]);
    fn remove(&self, name: &str);
}

/// A file in the writable store with the names of the asset files that the
/// game has removed, one on each line. The game cannot see this file.
const REMOVED: &str = ".removed";

pub struct Overlay<D, A> {
    data: D,
    assets: A,
}

impl<D: WritableStore, A: Store> Overlay<D, A> {
    pub fn new(data: D, assets: A) -> Self {
        Self { data, assets }
    }

    pub fn read(&self, name: &str) -> Option<Cow<'_, [u8]>> {
        if name == REMOVED {
            return None;
        }
        self.data.read(name).or_else(|| {
            if self.removed().iter().any(|n| n == name) {
                None
            } else {
                self.assets.read(name)
            }
        })
    }

    pub fn exists(&self, name: &str) -> bool {
        name != REMOVED
            && (self.data.contains(name)
                || (self.assets.contains(name) && !self.removed().iter().any(|n| n == name)))
    }

    /// The file goes to the writable store when the returned file is dropped.
    pub fn create(&self, name: &str) -> OverlayFile<'_, D, A> {
        OverlayFile {
            overlay: self,
            name: name.to_string(),
            content: Vec::new(),
        }
    }

    pub fn remove(&self, name: &str) {
        if self.data.contains(name) {
            self.data.remove(name);
        }
        if self.assets.contains(name) {
            let mut removed = self.removed();
            if !removed.iter().any(|n| n == name) {
                removed.push(name.to_string());
                self.write_removed(&removed);
            }
        }
        if trace::active() {
            trace::file_removed(name);
        }
    }

    /// The names that match a pattern of `FindFirst`, in alphabetical order
    pub fn find(&self, pattern: &str) -> Vec<String> {
        let removed = self.removed();
        let mut names: Vec<String> = self
            .data
            .names()
            .into_iter()
            .filter(|name| name != REMOVED)
            .chain(
                self.assets
                    .names()
                    .into_iter()
                    .filter(|name| !removed.contains(name)),
            )
            .filter(|name| matches(pattern.as_bytes(), name.as_bytes()))
            .collect();
        names.sort();
        names.dedup();
        names
    }

    fn close(&self, name: &str, content: &[u8]) {
        self.data.write(name, content);
        let mut removed = self.removed();
        if let Some(index) = removed.iter().position(|n| n == name) {
            removed.remove(index);
            self.write_removed(&removed);
        }
        if trace::active() {
            trace::file_written(name, content);
        }
    }

    fn removed(&self) -> Vec<String> {
        self.data
            .read(REMOVED)
            .map(|content| {
                String::from_utf8_lossy(&content)
                    .lines()
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn write_removed(&self, removed: &[String]) {
        if removed.is_empty() {
            if self.data.contains(REMOVED) {
                self.data.remove(REMOVED);
            }
        } else {
            let content: String = removed.iter().map(|name| format!("{name}\n")).collect();
            self.data.write(REMOVED, content.as_bytes());
        }
    }
}

pub struct OverlayFile<'a, D: WritableStore, A: Store> {
    overlay: &'a Overlay<D, A>,
    name: String,
    content: Vec<u8>,
}

impl<D: WritableStore, A: Store> Write for OverlayFile<'_, D, A> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.content.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<D: WritableStore, A: Store> Drop for OverlayFile<'_, D, A> {
    fn drop(&mut self) {
        self.overlay.close(&self.name, &self.content);
    }
}

#[derive(Default)]
pub struct MemoryStore {
    files: RefCell<BTreeMap<String, Vec<u8>>>,
}

impl Store for MemoryStore {
    fn read(&self, name: &str) -> Option<Cow<'_, [u8]>> {
        self.files.borrow().get(name).cloned().map(Cow::Owned)
    }
    fn names(&self) -> Vec<String> {
        self.files.borrow().keys().cloned().collect()
    }
}

impl WritableStore for MemoryStore {
    fn write(&self, name: &str, content: &[u8]) {
        self.files
            .borrow_mut()
            .insert(name.to_string(), content.to_vec());
    }
    fn remove(&self, name: &str) {
        self.files.borrow_mut().remove(name);
    }
}

/// `*` matches any characters and `?` matches one character. The case of the
/// letters must match, as in `FindFirst` of Free Pascal on Linux.
fn matches(pattern: &[u8], name: &[u8]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some((b'*', rest)) => (0..=name.len()).any(|skip| matches(rest, &name[skip..])),
        Some((&p, rest)) => match name.split_first() {
            Some((&n, name_rest)) => (p == b'?' || p == n) && matches(rest, name_rest),
            None => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn store(files: &[(&str, &str)]) -> MemoryStore {
        let store = MemoryStore::default();
        for (name, content) in files {
            store.write(name, content.as_bytes());
        }
        store
    }

    fn read(overlay: &Overlay<MemoryStore, MemoryStore>, name: &str) -> Option<String> {
        overlay.read(name).map(|content| {
            let mut s = String::new();
            content.as_ref().read_to_string(&mut s).unwrap();
            s
        })
    }

    #[test]
    fn reads_data_before_assets() {
        let overlay = Overlay::new(
            store(&[("CONFIG.SKI", "data")]),
            store(&[("CONFIG.SKI", "asset"), ("ANIM.SKI", "anim")]),
        );
        assert_eq!(read(&overlay, "CONFIG.SKI").as_deref(), Some("data"));
        assert_eq!(read(&overlay, "ANIM.SKI").as_deref(), Some("anim"));
        assert_eq!(read(&overlay, "NONE.SKI"), None);
    }

    #[test]
    fn writes_to_data() {
        let overlay = Overlay::new(store(&[]), store(&[("GOALS.SKI", "old")]));
        write!(overlay.create("GOALS.SKI"), "new").unwrap();
        assert_eq!(read(&overlay, "GOALS.SKI").as_deref(), Some("new"));
        assert_eq!(overlay.data.read("GOALS.SKI").as_deref(), Some(&b"new"[..]));
    }

    #[test]
    fn removing_an_asset_hides_it_until_it_is_written() {
        let overlay = Overlay::new(
            store(&[("A.SJH", "data")]),
            store(&[("A.SJH", "asset"), ("B.SJH", "asset")]),
        );
        overlay.remove("A.SJH");
        overlay.remove("B.SJH");
        assert!(!overlay.exists("A.SJH"));
        assert!(!overlay.exists("B.SJH"));
        assert_eq!(read(&overlay, "A.SJH"), None);
        assert!(overlay.find("*.SJH").is_empty());
        assert!(!overlay.exists(REMOVED));

        write!(overlay.create("B.SJH"), "new").unwrap();
        assert_eq!(read(&overlay, "B.SJH").as_deref(), Some("new"));
        assert_eq!(overlay.find("*"), ["B.SJH"]);
        overlay.remove("A.SJH");
        assert_eq!(overlay.data.names(), [REMOVED, "B.SJH"]);
    }

    #[test]
    fn finds_names_of_both_stores() {
        let overlay = Overlay::new(
            store(&[("C.SJH", ""), ("A.SJH", ""), ("X.SJR", "")]),
            store(&[("B.SJH", ""), ("A.SJH", ""), ("D.sjh", "")]),
        );
        assert_eq!(overlay.find("*.SJH"), ["A.SJH", "B.SJH", "C.SJH"]);
        assert_eq!(overlay.find("?.SJR"), ["X.SJR"]);
        assert!(overlay.find("*.SJC").is_empty());
    }

    #[test]
    fn matches_patterns() {
        assert!(matches(b"*.SJH", b"HILL.SJH"));
        assert!(matches(b"*.SJH", b".SJH"));
        assert!(!matches(b"*.SJH", b"HILL.SJHX"));
        assert!(!matches(b"*.SJH", b"HILL.sjh"));
        assert!(matches(b"NAMES?.SKI", b"NAMES1.SKI"));
        assert!(!matches(b"NAMES?.SKI", b"NAMES.SKI"));
        assert!(matches(b"*", b""));
    }
}
