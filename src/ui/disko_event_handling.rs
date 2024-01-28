use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{
    app::{Action, AppFocus},
    event_handling::DiskoEvent,
};

const SINGLE_KEY_COMMANDS_MAIN_SCREEN_COUNT: usize = 15;
const SINGLE_KEY_COMMANDS_CONFIRM_DELETE_POPUP_COUNT: usize = 9;
const MULTI_KEY_COMMANDS_COUNT: usize = 1;

const DEFAULT_SINGLE_KEY_COMMANDS_MAIN_SCREEN: [(KeyCode, Action);
    SINGLE_KEY_COMMANDS_MAIN_SCREEN_COUNT] = [
    (KeyCode::Esc, Action::Quit),
    (KeyCode::Char('q'), Action::Quit),
    (KeyCode::Char('s'), Action::ToggleSelection),
    (KeyCode::Char('d'), Action::ShowConfirmDeletePopup),
    (KeyCode::Down, Action::FocusNextItem),
    (KeyCode::Char('j'), Action::FocusNextItem),
    (KeyCode::Up, Action::FocusPreviousItem),
    (KeyCode::Char('k'), Action::FocusPreviousItem),
    (KeyCode::Char('G'), Action::FocusLastItem),
    (KeyCode::Right, Action::EnterFocusedDirectory),
    (KeyCode::Char('l'), Action::EnterFocusedDirectory),
    (KeyCode::Left, Action::EnterParentDirectory),
    (KeyCode::Char('h'), Action::EnterParentDirectory),
    (KeyCode::Char('a'), Action::SwitchEntryDisplaySize),
    (KeyCode::Char('b'), Action::SwitchProgress),
];

const DEFAULT_SINGLE_KEY_COMMANDS_CONFIRM_DELETE_POPUP: [(KeyCode, Action);
    SINGLE_KEY_COMMANDS_CONFIRM_DELETE_POPUP_COUNT] = [
    (KeyCode::Esc, Action::ShowMainScreen),
    (KeyCode::Char('q'), Action::ShowMainScreen),
    (KeyCode::Char('n'), Action::ShowMainScreen),
    (KeyCode::Right, Action::DeletePopupSwitchConfirmation),
    (KeyCode::Left, Action::DeletePopupSwitchConfirmation),
    (KeyCode::Char('l'), Action::DeletePopupSwitchConfirmation),
    (KeyCode::Char('h'), Action::DeletePopupSwitchConfirmation),
    (KeyCode::Enter, Action::DeletePopupSelect),
    (KeyCode::Char('y'), Action::ConfirmDelete),
];

const DEFAULT_MULTI_KEY_COMMANDS: [(&str, Action); MULTI_KEY_COMMANDS_COUNT] =
    [("gg", Action::FocusFirstItem(String::new()))];

pub struct DiskoEventHandler {
    buffer: Vec<char>,
    single_key_commands_main_screen: HashMap<KeyCode, Action>,
    single_key_commands_confirm_delete_popup: HashMap<KeyCode, Action>,
    multi_key_commands: HashMap<String, Action>,
}

impl Default for DiskoEventHandler {
    fn default() -> Self {
        let single_key_commands_main_screen =
            HashMap::from(DEFAULT_SINGLE_KEY_COMMANDS_MAIN_SCREEN);

        let single_key_commands_confirm_delete_popup =
            HashMap::from(DEFAULT_SINGLE_KEY_COMMANDS_CONFIRM_DELETE_POPUP);

        let multi_key_commands = HashMap::from(
            DEFAULT_MULTI_KEY_COMMANDS.map(|(command, action)| (command.to_string(), action)),
        );

        Self {
            buffer: Vec::new(),
            single_key_commands_main_screen,
            single_key_commands_confirm_delete_popup,
            multi_key_commands,
        }
    }
}

impl DiskoEventHandler {
    pub fn new(
        single_key_commands_main_screen: [(KeyCode, Action); SINGLE_KEY_COMMANDS_MAIN_SCREEN_COUNT],
        single_key_commands_confirm_delete_popup: [(KeyCode, Action);
            SINGLE_KEY_COMMANDS_CONFIRM_DELETE_POPUP_COUNT],
        multi_key_commands: [(&str, Action); MULTI_KEY_COMMANDS_COUNT],
    ) -> Self {
        let single_key_commands_main_screen = HashMap::from(single_key_commands_main_screen);

        let single_key_commands_confirm_delete_popup =
            HashMap::from(single_key_commands_confirm_delete_popup);

        let multi_key_commands = HashMap::from(
            multi_key_commands.map(|(command, action)| (command.to_string(), action)),
        );

        Self {
            buffer: Vec::new(),
            single_key_commands_main_screen,
            single_key_commands_confirm_delete_popup,
            multi_key_commands,
        }
    }

    /// Map the terminal event to an application action.
    pub fn handle_disko_events(&mut self, event: DiskoEvent, focus: &AppFocus) -> Option<Action> {
        // handle the events that are the same for all focus
        match event {
            DiskoEvent::Tick => Some(Action::Tick),
            DiskoEvent::TraversalFinished => Some(Action::SetTraversalFinished),
            DiskoEvent::Resize(w, h) => Some(Action::Resize(w, h)),
            DiskoEvent::Key(key) => match key.code {
                KeyCode::Char('c' | 'C') if key.modifiers == KeyModifiers::CONTROL => {
                    Some(Action::Quit)
                }
                _ => match focus {
                    AppFocus::MainScreen => self.handle_key_events_main_screen(key),
                    AppFocus::ConfirmDeletePopup(_) => {
                        self.handle_key_events_confirm_delete_popup(key)
                    }
                    AppFocus::BufferingInput => self.handle_key_events_buffering_input(key),
                },
            },
            _ => None,
        }
    }

    fn handle_key_events_main_screen(&mut self, key: KeyEvent) -> Option<Action> {
        let action = match key.modifiers {
            // SHIFT is needed to capture capitalized characters
            KeyModifiers::NONE | KeyModifiers::SHIFT => {
                self.single_key_commands_main_screen.get(&key.code)
            }
            // Other modifiers are ignored
            _ => return None,
        };

        // If a single key command is pressed, return the action
        if action.is_some() {
            return action.cloned();
        }

        self.handle_key_events_buffering_input(key)
    }

    fn handle_key_events_confirm_delete_popup(&self, key: KeyEvent) -> Option<Action> {
        match key.modifiers {
            // SHIFT is needed to capture capitalized characters
            // Only handle single key commands
            KeyModifiers::NONE | KeyModifiers::SHIFT => self
                .single_key_commands_confirm_delete_popup
                .get(&key.code)
                .cloned(),
            // Other modifiers are ignored
            _ => None,
        }
    }

    fn handle_key_events_buffering_input(&mut self, key: KeyEvent) -> Option<Action> {
        // SHIFT is needed to capture capitalized characters
        if matches!(key.modifiers, KeyModifiers::NONE | KeyModifiers::SHIFT) {
            if let KeyCode::Char(c) = key.code {
                return self.handle_buffered_input(c);
            }
        }

        // If any other modifier is used or a non-char key is pressed, clear the buffer
        self.buffer.clear();
        Some(Action::ShowMainScreen)
    }

    pub fn handle_buffered_input(&mut self, char: char) -> Option<Action> {
        self.buffer.push(char);
        let buffer_content = self.buffer.iter().collect::<String>();

        // If the buffer content is not a prefix of any multi key command, it is an invalid command
        if !self
            .multi_key_commands
            .keys()
            .any(|command| command.starts_with(&buffer_content))
        {
            self.buffer.clear();
            return Some(Action::InvalidInput(buffer_content));
        }

        match self.multi_key_commands.get(&buffer_content) {
            Some(action) => {
                self.buffer.clear();
                match action {
                    Action::FocusFirstItem(_) => Some(Action::FocusFirstItem(buffer_content)),
                    _ => None,
                }
            }
            // The buffer content is definitely a prefix of a multi key command
            None => Some(Action::BufferInput(buffer_content)),
        }
    }
}
