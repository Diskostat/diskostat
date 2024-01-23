use ratatui::widgets::TableState;

pub struct StatefulTable<T> {
    pub items: Vec<T>,
    pub state: TableState,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            items,
            state: TableState::default(),
        }
    }

    pub fn with_focused(items: Vec<T>, selected: Option<usize>) -> StatefulTable<T> {
        StatefulTable {
            items,
            state: TableState::default().with_selected(selected),
        }
    }

    pub fn toggle_selection(&mut self, index: usize) {
        if index > self.items.len() {
            return;
        }
        if self.selected.contains(&index) {
            self.selected.retain(|i| *i != index);
        } else {
            self.selected.push(index);
        }
    }

    pub fn focused_item(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    pub fn focus_next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn focus_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (self.items.len() + i - 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn focus_first(&mut self) {
        if self.items.is_empty() {
            return;
        }

        self.state.select(Some(0));
    }

    pub fn focus_last(&mut self) {
        if self.items.is_empty() {
            return;
        }

        self.state.select(Some(self.items.len() - 1));
    }
}
