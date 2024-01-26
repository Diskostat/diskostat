use filesize::PathExt;
use std::ops::{AddAssign, SubAssign};
use std::path::Path;

use byte_unit::Byte;

#[derive(Clone, Copy, Debug, Default)]
pub struct EntrySize {
    pub size: Byte,
    pub disc_size: Byte,
}

impl EntrySize {
    pub fn new(path: &Path, metadata: &std::fs::Metadata) -> Self {
        let disc_size = match path.size_on_disk_fast(metadata) {
            Ok(size) => Byte::from_u64(size),
            Err(_) => Byte::default(),
        };

        Self {
            size: Byte::from_u64(metadata.len()),
            disc_size,
        }
    }

    pub fn add(&mut self, size: EntrySize) {
        self.size.add(size.size);
        self.disc_size.add(size.disc_size);
    }

    pub fn sub(&mut self, size: EntrySize) {
        self.size.subtract(size.size);
        self.disc_size.subtract(size.disc_size);
    }
}

impl AddAssign for EntrySize {
    fn add_assign(&mut self, rhs: Self) {
        self.add(rhs);
    }
}

impl SubAssign for EntrySize {
    fn sub_assign(&mut self, rhs: Self) {
        self.sub(rhs);
    }
}
