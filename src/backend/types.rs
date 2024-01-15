use std::fs;

pub enum EnteryType {
    Directory,
    File(FileType),
}

pub enum FileType {
    Text,
    Binary,
    Image,
}

pub struct EnteryNode {
    name: String,
    size: usize,
    descendatns: usize,
    entery_type: EnteryType,
    medatata: fs::Metadata,
}
