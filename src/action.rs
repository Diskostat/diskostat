/// All possible application actions
pub enum Action {
    Tick,
    Resize(u16, u16),
    Quit,
}
