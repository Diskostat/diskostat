use std::fs::Metadata;

use super::{tree_walk_state::CustomJWalkClientState, entry_node::EntryNode, entry_type::{EntryType, FileType}};


impl EntryNode {
    /// Creates EntryNode from jwalk's DirEntry. That can be both file
    /// or dir. This funcion does not care about it.
    ///
    /// # Possible failures
    /// Metadata not found -> None
    /// OsString basename to String convertion fails -> None
    ///
    pub(crate) fn new(dir_entry: &jwalk::DirEntry<CustomJWalkClientState>) -> Option<Self> {
        let Some(metadata) = DirEntryToEntryNodeHelper::extract_metadata(dir_entry) else { return None };
        let Some(name) = DirEntryToEntryNodeHelper::extract_name(dir_entry) else { return None };
        let entry_type = DirEntryToEntryNodeHelper::extract_entry_type(dir_entry);
        // TODO: adjust!
        let size = metadata.len();

        Some(EntryNode{
            name,
            size,
            descendants_count: 0,
            entry_type,
            metadata,
        })
    }
}


/// Helper struct to extract EntryNode from from jwalk's DirEntry
struct DirEntryToEntryNodeHelper;

impl DirEntryToEntryNodeHelper {
    fn extract_name(dir_entry: &jwalk::DirEntry<CustomJWalkClientState>) -> Option<String> {
        match dir_entry.file_name.clone().into_string() {
            Ok(name) => Some(name),
            Err(os_string) => {
                dbg!("DirEntry has no name!\nOsString:", os_string,
                     "\nDirEntry:", dir_entry);
                None
            },
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
                dbg!("Error getting metadata from DirEntry:", dir_entry,
                     "\nerror: ", error);
                None
            }

        }
    }
}
