use filesize::PathExt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
pub struct EntrySize {
    pub apparent: u64,
    pub disk: u64,
}

impl EntrySize {
    pub fn new(path: &Path, metadata: &std::fs::Metadata) -> Self {
        Self {
            apparent: metadata.len(),
            disk: path.size_on_disk_fast(metadata).unwrap_or(0),
        }
    }
}

impl Add for EntrySize {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            apparent: self.apparent + other.apparent,
            disk: self.disk + other.disk,
        }
    }
}

impl Sub for EntrySize {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            apparent: self.apparent - other.apparent,
            disk: self.disk - other.disk,
        }
    }
}

impl AddAssign for EntrySize {
    fn add_assign(&mut self, rhs: Self) {
        *self = EntrySize {
            apparent: self.apparent + rhs.apparent,
            disk: self.disk + rhs.disk,
        };
    }
}

impl SubAssign for EntrySize {
    fn sub_assign(&mut self, rhs: Self) {
        *self = EntrySize {
            apparent: self.apparent - rhs.apparent,
            disk: self.disk - rhs.disk,
        };
    }
}
