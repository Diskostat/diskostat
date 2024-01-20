use std::{fs, path::Path, fmt::Display};

use super::entry_type::EntryType;


#[derive(Clone)]
pub(crate) struct EntryNode {
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) descendants_count: usize,
    pub(crate) entry_type: EntryType,
    pub(crate) metadata: fs::Metadata,
}

// Traits implementations

impl Display for EntryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntryNode: size={:12} | {}", self.size, self.name)
    }
}

// Convenience helpers

impl EntryNode {
    pub(crate) fn new_dir(path: &Path) -> Option<Self> {
        let Ok(metadata) = fs::metadata(path) else {
            dbg!("Failed to get metadata from path: ", path);
            return None
        };
        if !metadata.is_dir() { return None; }

        let name = path.file_name()
            .unwrap_or("no file_name".as_ref())
            .to_str().unwrap_or("no str")
            .to_string();

        return Some(Self {
            name,
            size: 0,
            descendants_count: 0,
            entry_type: EntryType::Directory,
            metadata,
        })
    }

}
