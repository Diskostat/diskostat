pub struct ConfirmDeletePopup {
    confirmed: bool,
}

impl ConfirmDeletePopup {
    pub fn new(confirmed: bool) -> Self {
        Self { confirmed }
    }

    pub fn confirmed(&self) -> bool {
        self.confirmed
    }

    pub fn switch_confirmation(&mut self) {
        self.confirmed = !self.confirmed;
    }
}
