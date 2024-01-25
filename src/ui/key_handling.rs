use crossterm::event::{KeyCode, KeyModifiers};

use super::{
    app::{Action, AppFocus},
    event_handling::Event,
};

/// Map the terminal event to an application action.
pub fn map_key_events(event: Event, focus: &AppFocus) -> Option<Action> {
    // handle the events that are the same for all foci
    let action = match event {
        Event::Tick => Some(Action::Tick),
        Event::Resize(w, h) => Some(Action::Resize(w, h)),
        Event::Key(key) => match key.code {
            KeyCode::Char('c' | 'C') if key.modifiers == KeyModifiers::CONTROL => {
                Some(Action::Quit)
            }
            _ => None,
        },
        _ => None,
    };
    if action.is_some() {
        return action;
    }
    match focus {
        AppFocus::MainScreen => map_key_events_main_screen(event),
        AppFocus::ConfirmDeletePopup(_) => map_key_events_confirm_delete_popup(event),
    }
}

fn map_key_events_main_screen(event: Event) -> Option<Action> {
    let action = match event {
        Event::Tick => Action::Tick,
        Event::Key(key) => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('s') => Action::ToggleSelection,
            KeyCode::Char('d') => Action::ShowConfirmDeletePopup,
            KeyCode::Down | KeyCode::Char('j') => Action::FocusNextItem,
            KeyCode::Up | KeyCode::Char('k') => Action::FocusPreviousItem,
            KeyCode::Char('g') => Action::FocusFirstItem,
            KeyCode::Char('G') => Action::FocusLastItem,
            KeyCode::Right | KeyCode::Char('l') => Action::EnterFocusedDirectory,
            KeyCode::Left | KeyCode::Char('h') => Action::EnterParentDirectory,
            _ => return None,
        },
        _ => return None,
    };
    Some(action)
}

fn map_key_events_confirm_delete_popup(event: Event) -> Option<Action> {
    let action = match event {
        Event::Tick => Action::Tick,
        Event::Key(key) => match key.code {
            KeyCode::Esc | KeyCode::Char('n' | 'q') => Action::ShowMainScreen,
            KeyCode::Right | KeyCode::Char('l' | 'h') | KeyCode::Left => {
                Action::DeletePopupSwitchConfirmation
            }
            KeyCode::Enter => Action::DeletePopupSelect,
            KeyCode::Char('y') => Action::ConfirmDelete,
            _ => return None,
        },
        _ => return None,
    };
    Some(action)
}
