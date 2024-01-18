
pub enum EntryType {
    Directory,
    File(FileType),
}

pub enum FileType {
    Text,
    Binary,
    Image,
}
