use std::{
    fmt::Display,
    fs::{self, Metadata},
    path::{Path, PathBuf},
};

use super::{
    entry_type::{EntryType, FileType},
    tree_walk_state::CustomJWalkClientState,
};

use byte_unit::Byte;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct EntryNode {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) size: u64,
    pub(crate) descendants_count: usize,
    pub(crate) entry_type: EntryType,
    pub(crate) metadata: fs::Metadata,
}

pub struct EntryNodeView {
    pub name: String,
    pub path: PathBuf,
    pub size: Byte,
    pub descendants_count: usize,
    pub entry_type: EntryType,
    pub index_to_original_node: Option<usize>,
}

impl EntryNodeView {
    pub fn new_dir(path: PathBuf) -> Self {
        Self {
            name: extract_file_name(&path),
            path,
            size: Byte::from_u64(0),
            descendants_count: 0,
            entry_type: EntryType::Directory,
            index_to_original_node: None,
        }
    }
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

        let name = extract_file_name(path);

        Some(Self {
            name,
            // TODO: Adjust! Get the real size of directory on disk
            // and/or its real size.
            path: path.to_path_buf(),
            size: 0,
            descendants_count: 0,
            entry_type: EntryType::Directory,
            metadata,
        })
    }
}

pub fn extract_file_name(path: &Path) -> String {
    if let Some(file_name) = path.file_name() {
        return file_name.to_string_lossy().to_string();
    }
    // If the path terminates in `..` then just set the path as the name.
    path.to_string_lossy().to_string()
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
        let name = value.file_name().to_string_lossy().to_string();
        let entry_type = Self::extract_entry_type(value);
        // TODO: Adjust! Get the real size of file on disk and or it's
        // real size.
        let size = metadata.len();

        Ok(EntryNode {
            name,
            path: value.path().clone(),
            size,
            descendants_count: 0,
            entry_type,
            metadata,
        })
    }
}

// Helper functions for TryFrom

impl EntryNode {
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
