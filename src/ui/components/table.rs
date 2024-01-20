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

    pub fn with_selected(items: Vec<T>, selected: Option<usize>) -> StatefulTable<T> {
        StatefulTable {
            items,
            state: TableState::default().with_selected(selected),
        }
    }

    pub fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select_first(&mut self) {
        self.state.select(Some(0));
    }

    pub fn select_last(&mut self) {
        self.state.select(Some(self.items.len() - 1));
    }
}
