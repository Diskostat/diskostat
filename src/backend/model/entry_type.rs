

#[derive(Clone, Copy)]
pub enum EntryType {
    Directory,
    File(FileType),
}

#[derive(Clone, Copy)]
pub enum FileType {
    Text,
    Binary,
    Image,
}
