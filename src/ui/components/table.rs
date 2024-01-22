use ratatui::widgets::TableState;

pub struct StatefulTable<T> {
    pub items: Vec<T>,
    pub state: TableState,
    pub selected: Vec<usize>,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            items,
            state: TableState::default(),
            selected: Vec::default(),
        }
    }

    pub fn with_focused(items: Vec<T>, focused: Option<usize>) -> StatefulTable<T> {
        StatefulTable {
            items,
            state: TableState::default().with_selected(focused),
            selected: Vec::default(),
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

    pub fn focused(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    pub fn focused_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.selected.contains(&index)
    }

    pub fn clear_selected(&mut self) {
        self.selected.clear();
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
