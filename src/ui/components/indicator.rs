/// ["|", "/", "-", "\\"]
pub const ASCII: &[char] = &['|', '/', '-', '\\'];

/// ["◷", "◶", "◵", "◴"]
pub const WHITE_CIRCLE: &[char] = &['◷', '◶', '◵', '◴'];

/// ["⠘", "⠰", "⠤", "⠆", "⠃", "⠉"]
pub const BRAILLE_DOUBLE: &[char] = &['⠘', '⠰', '⠤', '⠆', '⠃', '⠉'];

pub struct Indicator {
    pub symbols: Vec<char>,
    pub label: String,
    index: usize,
}

impl Indicator {
    pub fn new(symbols: &[char], label: String) -> Self {
        Self {
            symbols: Vec::from(symbols),
            label,
            index: 0,
        }
    }

    pub fn next_step(&mut self) {
        self.index = (self.index + 1) % self.symbols.len();
    }

    pub fn render(&self) -> String {
        format!("{} {}", self.symbols[self.index], self.label)
    }
}
