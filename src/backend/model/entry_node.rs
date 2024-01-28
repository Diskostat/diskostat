use std::{
    fmt::Display,
    fs::{self, Metadata},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};

use super::{entry_size::EntrySize, tree_walk_state::CustomJWalkClientState};

#[derive(Clone, Copy, Debug)]
pub enum EntryType {
    Directory,
    File,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct EntryNode {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) sizes: EntrySize,
    pub(crate) dir_size: Option<EntrySize>,
    pub(crate) descendants_count: usize,
    pub(crate) entry_type: EntryType,
    pub(crate) metadata: fs::Metadata,
}

pub struct EntryNodeView {
    pub name: String,
    pub path: PathBuf,
    pub sizes: EntrySize,
    pub dir_size: Option<EntrySize>,
    pub descendants_count: usize,
    pub entry_type: EntryType,
    pub mode: Mode,
    pub access_time: Option<DateTime<Local>>,
    pub index_to_original_node: Option<usize>,
}

pub enum Mode {
    Permissions(u32),
    Attributes(u32),
    Unknown,
}

impl EntryNodeView {
    pub(crate) fn new_dir(path: PathBuf) -> Self {
        Self {
            name: extract_file_name(&path),
            path,
            sizes: EntrySize::default(),
            dir_size: Some(EntrySize::default()),
            descendants_count: 0,
            entry_type: EntryType::Directory,
            // Unknown here for now, this needs to be updated later during the
            // backend refactor.
            mode: Mode::Unknown,
            access_time: None,
            index_to_original_node: None,
        }
    }

    pub(crate) fn from_entry_node(entry_node: &EntryNode) -> Self {
        Self {
            name: entry_node.name.clone(),
            path: entry_node.path.clone(),
            sizes: entry_node.sizes,
            dir_size: entry_node.dir_size,
            descendants_count: entry_node.descendants_count,
            entry_type: entry_node.entry_type,
            access_time: entry_node
                .metadata
                .accessed()
                .ok()
                .map(DateTime::<Local>::from),
            mode: extract_mode(&entry_node.metadata),
            index_to_original_node: None,
        }
    }
}

// Convenience helpers

impl EntryNode {
    pub(crate) fn new_dir(path: &Path) -> Option<(Self, EntrySize)> {
        let Ok(metadata) = fs::metadata(path) else {
            return None;
        };
        if !metadata.is_dir() {
            return None;
        }

        let name = extract_file_name(path);
        let size = EntrySize::new(path, &metadata);

        Some((
            Self {
                name,
                path: path.to_path_buf(),
                sizes: EntrySize::default(),
                dir_size: Some(size),
                descendants_count: 0,
                entry_type: EntryType::Directory,
                metadata,
            },
            size,
        ))
    }

    pub(crate) fn delete_entry(&self) -> std::io::Result<()> {
        match self.entry_type {
            EntryType::Directory => std::fs::remove_dir_all(self.path.clone()),
            EntryType::File => std::fs::remove_file(self.path.clone()),
        }
    }
}

fn extract_file_name(path: &Path) -> String {
    if let Some(file_name) = path.file_name() {
        return file_name.to_string_lossy().to_string();
    }
    // If the path terminates in `..` then just set the path as the name.
    path.to_string_lossy().to_string()
}

#[cfg(windows)]
fn extract_mode(metadata: &Metadata) -> Mode {
    use std::os::windows::fs::MetadataExt;
    let attributes = metadata.file_attributes();
    Mode::Attributes(attributes)
}

#[cfg(unix)]
fn extract_mode(metadata: &Metadata) -> Mode {
    use std::os::unix::fs::MetadataExt;
    let mode = metadata.mode();
    Mode::Permissions(mode)
}

#[cfg(not(any(unix, windows)))]
pub fn extract_mode(_metadata: &Metadata) -> Mode {
    Mode::Unknown
}

// Traits implementations

impl Display for EntryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<20} • {}", self.name, self.sizes.apparent_size)
    }
}

impl TryFrom<&jwalk::DirEntry<CustomJWalkClientState>> for EntryNode {
    type Error = &'static str;

    fn try_from(value: &jwalk::DirEntry<CustomJWalkClientState>) -> Result<Self, Self::Error> {
        let Ok(metadata) = value.metadata() else {
            return Err("Error getting metadata from DirEntry");
        };
        let name = value.file_name().to_string_lossy().to_string();
        let entry_type = Self::extract_entry_type(value);

        let size = EntrySize::new(value.path().as_path(), &metadata);
        let dir_size = match entry_type {
            EntryType::Directory => Some(size),
            EntryType::File => None,
        };

        Ok(EntryNode {
            name,
            path: value.path().clone(),
            sizes: size,
            dir_size,
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
        EntryType::File
    }
}
