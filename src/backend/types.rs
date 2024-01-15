use std::fs;

pub enum EntryType {
    Directory,
    File(FileType),
}

pub enum FileType {
    Text,
    Binary,
    Image,
}

pub struct EntryNode {
    name: String,
    size: usize,
    descendants: usize,
    entry_type: EntryType,
    medatata: fs::Metadata,
}
