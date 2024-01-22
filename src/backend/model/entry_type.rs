#[derive(Clone, Copy, Debug)]
pub enum EntryType {
    Directory,
    File(FileType),
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum FileType {
    Text,
    Binary,
    Image,
}
