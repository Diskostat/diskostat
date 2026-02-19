use std::{
    cell::OnceCell,
    fmt::Display,
    fs::{self, Metadata},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};

use super::entry_size::EntrySize;

#[derive(Clone, Copy, Debug)]
pub enum EntryType {
    Directory,
    File,
}

impl From<&Metadata> for EntryType {
    fn from(metadata: &Metadata) -> Self {
        if metadata.is_dir() {
            return EntryType::Directory;
        }
        EntryType::File
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct EntryNode {
    pub(crate) path: PathBuf,
    pub(crate) size: EntrySize,
    pub(crate) descendants_count: usize,
    pub(crate) entry_type: EntryType,
}

pub struct EntryNodeView {
    pub name: String,
    pub path: PathBuf,
    pub size: EntrySize,
    pub descendants_count: usize,
    pub entry_type: EntryType,
    details: OnceCell<Option<EntryDetails>>,
    pub index_to_original_node: Option<usize>,
}

pub struct EntryDetails {
    mode: Option<Mode>,
    self_size: EntrySize,
    access_time: Option<DateTime<Local>>,
}

impl EntryDetails {
    pub fn new(metadata: &Metadata, path: &Path) -> Self {
        Self {
            mode: extract_mode(metadata),
            self_size: EntrySize::new(path, metadata),
            access_time: metadata.accessed().ok().map(DateTime::<Local>::from),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Mode {
    Permissions(u32),
    Attributes(u32),
}

impl EntryNodeView {
    pub(crate) fn new_dir(path: PathBuf) -> Self {
        Self {
            name: extract_file_name(&path),
            path,
            size: EntrySize::default(),
            descendants_count: 0,
            entry_type: EntryType::Directory,
            details: OnceCell::new(),
            index_to_original_node: None,
        }
    }

    fn details(&self) -> Option<&EntryDetails> {
        self.details
            .get_or_init(|| {
                let metadata = &fs::metadata(&self.path).ok()?;
                Some(EntryDetails::new(metadata, &self.path))
            })
            .as_ref()
    }

    pub(crate) fn self_size(&self) -> EntrySize {
        self.details().map(|d| d.self_size).unwrap_or_default()
    }

    pub(crate) fn access_time(&self) -> Option<DateTime<Local>> {
        self.details().and_then(|d| d.access_time)
    }

    pub(crate) fn mode(&self) -> Option<Mode> {
        self.details().and_then(|d| d.mode)
    }
}

impl From<&EntryNode> for EntryNodeView {
    fn from(entry_node: &EntryNode) -> Self {
        Self {
            name: extract_file_name(&entry_node.path),
            path: entry_node.path.clone(),
            size: entry_node.size,
            descendants_count: entry_node.descendants_count,
            entry_type: entry_node.entry_type,
            details: OnceCell::new(),
            index_to_original_node: None,
        }
    }
}

// Convenience helpers

impl EntryNode {
    pub(crate) fn new(path: PathBuf, metadata: &Metadata) -> Self {
        let size = EntrySize::new(&path, metadata);
        Self {
            path,
            size,
            descendants_count: 0,
            entry_type: metadata.into(),
        }
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

#[cfg(any(unix, windows))]
impl From<&Metadata> for Mode {
    #[cfg(windows)]
    fn from(metadata: &Metadata) -> Self {
        use std::os::windows::fs::MetadataExt;
        Mode::Attributes(metadata.file_attributes())
    }

    #[cfg(unix)]
    fn from(metadata: &Metadata) -> Self {
        use std::os::unix::fs::MetadataExt;
        Mode::Permissions(metadata.mode())
    }
}

#[cfg(any(unix, windows))]
fn extract_mode(metadata: &Metadata) -> Option<Mode> {
    Some(metadata.into())
}

#[cfg(not(any(unix, windows)))]
fn extract_mode(_metadata: &Metadata) -> Option<Mode> {
    None
}

// Traits implementations

impl Display for EntryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<20} • {}",
            extract_file_name(&self.path),
            self.size.apparent
        )
    }
}
