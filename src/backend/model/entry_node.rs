use std::{
    fmt::Display,
    fs::{self, Metadata},
    path::Path,
};

use super::{
    entry_type::{EntryType, FileType},
    tree_walk_state::CustomJWalkClientState,
};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct EntryNode {
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) descendants_count: usize,
    pub(crate) entry_type: EntryType,
    pub(crate) metadata: fs::Metadata,
}

// Convenience helpers

impl EntryNode {
    pub(crate) fn new_dir(path: &Path) -> Option<Self> {
        let Ok(metadata) = fs::metadata(path) else {
            dbg!("Failed to get metadata from path: ", path);
            return None;
        };
        if !metadata.is_dir() {
            return None;
        }

        let name = path
            .file_name()
            // `std::path::Path` returns None, if the path refers to
            // the parent directory (..). Therefore, we create that
            // manually.
            //
            // TODO: Check if .. is / and corrent the name if so.
            .unwrap_or("..".as_ref())
            // See docs for usage.
            .to_string_lossy()
            .to_string();

        Some(Self {
            name,
            // TODO: Adjust! Get the real size of directory on disk
            // and/or its real size.
            size: 0,
            descendants_count: 0,
            entry_type: EntryType::Directory,
            metadata,
        })
    }
}

// Traits implementations

impl Display for EntryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<20} • {}", self.name, self.size)
    }
}

impl TryFrom<&jwalk::DirEntry<CustomJWalkClientState>> for EntryNode {
    type Error = &'static str;

    fn try_from(value: &jwalk::DirEntry<CustomJWalkClientState>) -> Result<Self, Self::Error> {
        let Some(metadata) = Self::extract_metadata(value) else {
            return Err("Error getting metadata from DirEntry");
        };
        let Some(name) = Self::extract_name(value) else {
            return Err("Error getting name from DirEntry");
        };
        let entry_type = Self::extract_entry_type(value);
        // TODO: Adjust! Get the real size of file on disk and or it's
        // real size.
        let size = metadata.len();

        Ok(EntryNode {
            name,
            size,
            descendants_count: 0,
            entry_type,
            metadata,
        })
    }
}

// Helper functions for TryFrom

impl EntryNode {
    fn extract_name(dir_entry: &jwalk::DirEntry<CustomJWalkClientState>) -> Option<String> {
        match dir_entry.file_name.clone().into_string() {
            Ok(name) => Some(name),
            Err(os_string) => {
                dbg!(
                    "DirEntry has no name!\nOsString:",
                    os_string,
                    "\nDirEntry:",
                    dir_entry
                );
                None
            }
        }
    }

    fn extract_entry_type(dir_entry: &jwalk::DirEntry<CustomJWalkClientState>) -> EntryType {
        if dir_entry.file_type.is_dir() {
            return EntryType::Directory;
        }
        EntryType::File(FileType::Text)
    }

    fn extract_metadata(dir_entry: &jwalk::DirEntry<CustomJWalkClientState>) -> Option<Metadata> {
        match dir_entry.metadata() {
            Ok(metadata) => Some(metadata),
            Err(error) => {
                dbg!(
                    "Error getting metadata from DirEntry:",
                    dir_entry,
                    "\nerror: ",
                    error
                );
                None
            }
        }
    }
}
