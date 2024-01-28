use ratatui::widgets::TableState;

use std::collections::HashSet;

pub struct StatefulTable<T> {
    pub items: Vec<T>,
    pub selected: HashSet<usize>,
    pub state: TableState,
}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
        StatefulTable {
            items,
            state: TableState::default(),
            selected: HashSet::default(),
        }
    }

    pub fn with_focused(items: Vec<T>, focused: Option<usize>) -> StatefulTable<T> {
        StatefulTable {
            items,
            state: TableState::default().with_selected(focused),
            selected: HashSet::default(),
        }
    }

    pub fn toggle_selection(&mut self, index: usize) {
        if index > self.items.len() {
            return;
        }
        let was_present = self.selected.remove(&index);
        if !was_present {
            self.selected.insert(index);
        }
    }

    pub fn focused(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    pub fn is_focused(&self, index: usize) -> bool {
        self.state.selected() == Some(index)
    }

    pub fn focused_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected(&self) -> Vec<&T> {
        self.selected
            .iter()
            .filter_map(|i| self.items.get(*i))
            .collect()
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
