

#[derive(Clone, Copy, Debug)]
pub enum EntryType {
    Directory,
    File(FileType),
}

#[derive(Clone, Copy, Debug)]
pub enum FileType {
    Text,
    Binary,
    Image,
}
