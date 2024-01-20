use crossterm::event::{KeyCode, KeyModifiers};

use super::{app::Action, event_handling::Event};

/// Map the terminal event to an application action.
pub fn map_key_events(event: Event) -> Option<Action> {
    let action = match event {
        Event::Tick => Action::Tick,
        Event::Key(key) => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('c') | KeyCode::Char('C') if key.modifiers == KeyModifiers::CONTROL => {
                Action::Quit
            }
            KeyCode::Down | KeyCode::Char('j') => Action::FocusNextItem,
            KeyCode::Up | KeyCode::Char('k') => Action::FocusPreviousItem,
            KeyCode::Char('g') => Action::FocusFirstItem,
            KeyCode::Char('G') => Action::FocusLastItem,
            _ => return None,
        },
        Event::Mouse(_) => return None,
        Event::Resize(w, h) => Action::Resize(w, h),
        _ => return None,
    };
    Some(action)
}
