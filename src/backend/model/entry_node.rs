use std::{
    fmt::Display,
    fs::{self, Metadata},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};

use super::{
    entry_size::EntrySize,
    entry_type::{EntryType, FileType},
    tree_walk_state::CustomJWalkClientState,
};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct EntryNode {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) sizes: EntrySize,
    pub(crate) descendants_count: usize,
    pub(crate) entry_type: EntryType,
    pub(crate) metadata: fs::Metadata,
}

pub struct EntryNodeView {
    pub name: String,
    pub path: PathBuf,
    pub sizes: EntrySize,
    pub descendants_count: usize,
    pub entry_type: EntryType,
    pub mode: Mode,
    pub access_time: Option<DateTime<Local>>,
    pub index_to_original_node: Option<usize>,
}

pub enum Mode {
    Permissions(String),
    Attributes(String),
    Unknown,
}

impl EntryNodeView {
    pub(crate) fn new_dir(path: PathBuf) -> Self {
        Self {
            name: extract_file_name(&path),
            path,
            sizes: EntrySize::default(),
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
            dbg!("Failed to get metadata from path: ", path);
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
            EntryType::File(_) => std::fs::remove_file(self.path.clone()),
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
    let mut result = String::new();
    // https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
    if attributes & 0x00000010 == 0 {
        result.push_str("-");
    } else {
        result.push_str("d");
    };
    if attributes & 0x00000020 == 0 {
        result.push_str("-");
    } else {
        result.push_str("a");
    };
    if attributes & 0x00000001 == 0 {
        result.push_str("-");
    } else {
        result.push_str("r");
    };
    if attributes & 0x00000002 == 0 {
        result.push_str("-");
    } else {
        result.push_str("h");
    };
    if attributes & 0x00000004 == 0 {
        result.push_str("-");
    } else {
        result.push_str("s");
    };
    Mode::Attributes(result)
}

#[cfg(unix)]
fn get_access_string_triple(octal: u32) -> String {
    let mut result = String::new();
    result.push_str(if octal & 0o4 == 0 { "-" } else { "r" });
    result.push_str(if octal & 0o2 == 0 { "-" } else { "w" });
    result.push_str(if octal & 0o1 == 0 { "-" } else { "x" });
    result
}

#[cfg(unix)]
fn extract_mode(metadata: &Metadata) -> Mode {
    use std::os::unix::fs::MetadataExt;
    let mode = metadata.mode();
    let mut user = get_access_string_triple(mode >> 6);
    let mut group = get_access_string_triple(mode >> 3);
    let mut others = get_access_string_triple(mode);

    // SUID
    if mode & 0o4000 != 0 {
        if mode & 0o100 == 0 {
            user.replace_range(2..3, "S");
        } else {
            user.replace_range(2..3, "s");
        }
    }
    // SGID
    if mode & 0o2000 != 0 {
        if mode & 0o010 == 0 {
            group.replace_range(2..3, "S");
        } else {
            group.replace_range(2..3, "s");
        }
    }
    // Sticky
    if mode & 0o1000 != 0 {
        others.replace_range(2..3, "t");
    }

    let masked = mode & 0o170000;

    // https://man7.org/linux/man-pages/man7/inode.7.html
    let file_type = if masked == 0o140000 {
        "s"
    } else if masked == 0o120000 {
        "l"
    } else if masked == 0o100000 {
        "-"
    } else if masked == 0o060000 {
        "b"
    } else if masked == 0o040000 {
        "d"
    } else if masked == 0o020000 {
        "c"
    } else if masked == 0o010000 {
        "p"
    // Should not happen.
    } else {
        "?"
    };

    Mode::Permissions(format!("{file_type}{user}{group}{others}"))
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
        let Some(metadata) = Self::extract_metadata(value) else {
            return Err("Error getting metadata from DirEntry");
        };
        let name = value.file_name().to_string_lossy().to_string();
        let entry_type = Self::extract_entry_type(value);

        Ok(EntryNode {
            name,
            path: value.path().clone(),
            sizes: EntrySize::new(value.path().as_path(), &metadata),
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
