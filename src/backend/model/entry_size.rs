use filesize::PathExt;
use std::ops::{AddAssign, SubAssign};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
pub struct EntrySize {
    pub apparent_size: u64,
    pub disk_size: u64,
}

impl EntrySize {
    pub fn new(path: &Path, metadata: &std::fs::Metadata) -> Self {
        Self {
            apparent_size: metadata.len(),
            disk_size: path.size_on_disk_fast(metadata).unwrap_or(0),
        }
    }
}

impl AddAssign for EntrySize {
    fn add_assign(&mut self, rhs: Self) {
        *self = EntrySize {
            apparent_size: self.apparent_size + rhs.apparent_size,
            disk_size: self.disk_size + rhs.disk_size,
        };
    }
}

impl SubAssign for EntrySize {
    fn sub_assign(&mut self, rhs: Self) {
        *self = EntrySize {
            apparent_size: self.apparent_size - rhs.apparent_size,
            disk_size: self.disk_size - rhs.disk_size,
        };
    }
}
