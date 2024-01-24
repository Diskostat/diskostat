pub struct ConfirmDeletePopup {
    selected_yes: bool,
}

impl ConfirmDeletePopup {
    pub fn new(selected_yes: bool) -> Self {
        Self { selected_yes }
    }

    pub fn selected_yes(&self) -> bool {
        self.selected_yes
    }

    pub fn tab(&mut self) {
        self.selected_yes = !self.selected_yes;
    }
}
