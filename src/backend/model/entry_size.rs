use std::ops::{AddAssign, SubAssign};

use byte_unit::Byte;

#[derive(Clone, Copy, Debug, Default)]
pub struct EntrySize {
    pub size: Byte,
    pub disc_size: Byte,
}

impl EntrySize {
    pub fn new(metadata: std::fs::Metadata) -> Self {
        Self {
            size: Byte::from_u64(metadata.len()),
            // TODO: Calculate disc size.
            disc_size: Byte::from_u64(metadata.len()),
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

// TODO: Display?
