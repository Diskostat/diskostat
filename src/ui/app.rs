use std::{fs::File, io::Read, sync::mpsc};

use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::backend::{
    disko_tree::DiskoTree,
    model::{entry_node::EntryNodeView, entry_type::EntryType},
};

use super::{
    color_theme::ColorTheme,
    components::{confirm_delete::ConfirmDeletePopup, indicator, table::StatefulTable},
    disko_event_handling::DiskoEventHandler,
    event_handling::{DiskoEvent, EventHandler},
    renderer,
    tui::Tui,
};

use anyhow::Result;

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

const CLEAR_MESSAGE_AFTER_SECONDS: u64 = 2;

/// All possible application actions.
#[derive(Clone)]
pub enum Action {
    Tick,
    Quit,
    SetTraversalFinished,
    Resize(u16, u16),
    ShowMainScreen,
    ShowConfirmDeletePopup,
    BufferInput(String),
    InvalidInput(String),
    FocusNextItem,
    FocusPreviousItem,
    FocusFirstItem(String),
    FocusLastItem,
    EnterFocusedDirectory,
    EnterParentDirectory,
    DeletePopupSwitchConfirmation,
    DeletePopupSelect,
    ConfirmDelete,
    ToggleSelection,
    SwitchProgress,
}

/// Possible application preview states.
pub enum Preview {
    Text(String),
    Table(StatefulTable<EntryNodeView>),
    EmptyDirectory,
    Empty,
}

pub enum AppFocus {
    MainScreen,
    ConfirmDeletePopup(ConfirmDeletePopup),
    BufferingInput,
}

/// Application state.
pub struct AppState {
    pub should_quit: bool,
    pub main_table: StatefulTable<EntryNodeView>,
    pub preview: Preview,
    pub focus: AppFocus,
    pub current_directory: EntryNodeView,
    pub traversal_finished: bool,
    pub show_bar: bool,
    pub message: String,
    pub clear_message_ticks: u64,
    pub indicator: indicator::Indicator,
}

/// Application.
pub struct App {
    state: AppState,
    tui: Tui,
    disko_events: DiskoEventHandler,
    tree: DiskoTree,
    tick_rate: u64,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(tick_rate: u64, render_rate: u64, tree: DiskoTree) -> Result<Self> {
        // Initialize the terminal user interface.
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;

        let (sender, receiver) = mpsc::channel();
        let events = EventHandler::new(tick_rate, render_rate, sender, receiver);

        let renderer = renderer::Renderer::new(ColorTheme::default());
        let tui = Tui::new(terminal, events, renderer);

        let state = AppState {
            should_quit: false,
            main_table: StatefulTable::with_focused(vec![], None),
            preview: Preview::Empty,
            focus: AppFocus::MainScreen,
            current_directory: EntryNodeView::new_dir(tree.root_path()),
            traversal_finished: false,
            show_bar: false,
            message: String::new(),
            clear_message_ticks: 0,
            indicator: indicator::Indicator::new(indicator::ASCII, "Traversing".to_string()),
        };

        let disko_events = DiskoEventHandler::default();

        Ok(Self {
            state,
            tui,
            disko_events,
            tree,
            tick_rate,
        })
    }

    fn get_file_preview(&self, entry: &EntryNodeView) -> Preview {
        let Ok(file) = File::open(&entry.path) else {
            return Preview::Empty;
        };
        let bytes = 3200;
        let mut buffer = String::with_capacity(bytes);
        if file.take(bytes as u64).read_to_string(&mut buffer).is_ok() {
            return Preview::Text(buffer);
        };
        // If we can't read the file, assume it is not a text file and don't
        // show anything.
        Preview::Empty
    }

    fn get_directory_preview(&self, entry: &EntryNodeView) -> Preview {
        let subdir_entries = self
            .tree
            .get_subdir_of_current_dir_view(
                entry
                    .index_to_original_node
                    .expect("should never get the root directory as a child"),
            )
            .expect("child directory at the given index should exist");

        if subdir_entries.is_empty() {
            return Preview::EmptyDirectory;
        }

        Preview::Table(StatefulTable::with_items(subdir_entries))
    }

    /// Runs the main loop of the application.
    pub fn run(&mut self) -> Result<()> {
        self.tui.enter()?;
        let sender = self.tui.events.get_event_sender();
        self.tree.start_background_traversal(sender);

        // Start the main loop.
        while !self.state.should_quit {
            let event = self.tui.events.next()?;

            // Render the user interface.
            if let DiskoEvent::Init | DiskoEvent::Render = event {
                self.tui.draw(&mut self.state)?;
            }

            // Handle events.
            let action = self
                .disko_events
                .handle_disko_events(event, &self.state.focus);

            self.update(action)?;
        }

        self.tree.stop_background_traversal();

        // Exit the user interface.
        self.tui.exit()?;
        Ok(())
    }

    pub fn update_view_on_switch_dir(&mut self) {
        let Some((current_directory, entries)) = self.tree.get_current_dir_view() else {
            return;
        };
        self.state.current_directory = current_directory;
        let focused_index = {
            if entries.is_empty() {
                None
            } else {
                Some(0)
            }
        };
        self.state.main_table = StatefulTable::with_focused(entries, focused_index);
        self.update_focus();
    }

    pub fn update_view(&mut self) {
        let Some((current_directory, entries)) = self.tree.get_current_dir_view() else {
            return;
        };
        self.state.current_directory = current_directory;
        let focused_index = {
            if entries.is_empty() {
                None
            }
            // If there was a focus prior, try to focus the same entry again.
            // It's possible that there were multiple entries removed, so the focus
            // is out of bounds now. In that case, focus the last entry.
            else if let Some(index) = self.state.main_table.focused_index() {
                Some(index.min(entries.len() - 1))
            }
            // There was no focus prior, so focus the first entry.
            else {
                Some(0)
            }
        };
        self.state.main_table = StatefulTable::with_focused(entries, focused_index);
        self.update_focus();
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        if !matches!(self.state.focus, AppFocus::BufferingInput)
            && self.state.clear_message_ticks >= self.tick_rate * CLEAR_MESSAGE_AFTER_SECONDS
        {
            self.clear_message();
        } else {
            self.state.clear_message_ticks += 1;
        }

        if self.state.traversal_finished {
            return;
        }
        self.state.indicator.next_step();
        self.update_view();
    }

    /// Handles the resize event of the terminal.
    pub fn resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.tui.terminal.resize(Rect::new(0, 0, width, height))?;
        Ok(())
    }

    /// Set the application to quit.
    pub fn quit(&mut self) {
        self.state.should_quit = true;
    }

    fn update_focus(&mut self) {
        let Some(focused) = self.state.main_table.focused() else {
            self.state.preview = Preview::Empty;
            return;
        };
        self.state.preview = match focused.entry_type {
            EntryType::File(_) => self.get_file_preview(focused),
            EntryType::Directory => self.get_directory_preview(focused),
        };
    }

    fn set_message(&mut self, message: String) {
        self.state.message = message.to_string();
        self.state.clear_message_ticks = 0;
    }

    fn clear_message(&mut self) {
        self.state.message.clear();
        self.state.clear_message_ticks = 0;
    }

    /// Handle the application actions.
    pub fn update(&mut self, action: Option<Action>) -> Result<()> {
        if let Some(action) = action {
            match action {
                Action::Tick => self.tick(),
                Action::Quit => self.quit(),
                Action::SetTraversalFinished => {
                    self.set_message("Traversal finished".to_string());
                    self.state.traversal_finished = true;
                    self.update_view();
                }
                Action::Resize(w, h) => self.resize(w, h)?,
                Action::ShowMainScreen => {
                    self.state.focus = AppFocus::MainScreen;
                }
                Action::ShowConfirmDeletePopup => {
                    if !self.state.traversal_finished {
                        self.set_message("Cannot delete while traversing".to_string());
                        return Ok(());
                    }

                    self.state.focus = AppFocus::ConfirmDeletePopup(ConfirmDeletePopup::new(true));
                }
                Action::BufferInput(input) => {
                    self.set_message(input);
                    self.state.focus = AppFocus::BufferingInput;
                }
                Action::InvalidInput(input) => {
                    self.set_message(format!("Invalid input: {}", input));
                    self.state.focus = AppFocus::MainScreen;
                }
                Action::FocusNextItem => {
                    self.state.main_table.focus_next();
                    self.update_focus();
                }
                Action::FocusPreviousItem => {
                    self.state.main_table.focus_previous();
                    self.update_focus();
                }
                Action::FocusFirstItem(input) => {
                    self.set_message(input);
                    self.state.main_table.focus_first();
                    self.state.focus = AppFocus::MainScreen;
                    self.update_focus();
                }
                Action::FocusLastItem => {
                    self.state.main_table.focus_last();
                    self.update_focus();
                }
                Action::DeletePopupSwitchConfirmation => {
                    if let AppFocus::ConfirmDeletePopup(popup) = &mut self.state.focus {
                        popup.switch_confirmation();
                    }
                }
                Action::DeletePopupSelect => {
                    if let AppFocus::ConfirmDeletePopup(popup) = &mut self.state.focus {
                        if popup.confirmed() {
                            self.delete_selected();
                        }
                        self.state.focus = AppFocus::MainScreen;
                    }
                }
                Action::ConfirmDelete => {
                    self.delete_selected();
                    self.state.focus = AppFocus::MainScreen;
                }
                Action::ToggleSelection => {
                    if !self.state.traversal_finished {
                        self.set_message("Cannot select while traversing".to_string());
                        return Ok(());
                    }

                    if let Some(focused) = self.state.main_table.focused_index() {
                        self.state.main_table.toggle_selection(focused);
                    };
                }
                Action::EnterFocusedDirectory => {
                    if let Some(focused) = self.state.main_table.focused() {
                        if !matches!(focused.entry_type, EntryType::Directory) {
                            return Ok(());
                        }
                        if self
                            .tree
                            .switch_to_subdirectory(
                                focused
                                    .index_to_original_node
                                    .expect("root should never be focused"),
                            )
                            .is_ok()
                        {
                            self.update_view_on_switch_dir();
                            self.state.main_table.clear_selected();
                        }
                    }
                }
                Action::EnterParentDirectory => {
                    // Ignore if there is no parent anymore.
                    if self.tree.switch_to_parent_directory().is_ok() {
                        self.update_view_on_switch_dir();
                        self.state.main_table.clear_selected();
                    }
                }
                Action::SwitchProgress => self.state.show_bar = !self.state.show_bar,
            }
        }
        Ok(())
    }

    pub fn delete_selected(&mut self) {
        let mut indeces: Vec<usize> = self
            .state
            .main_table
            .selected
            .iter()
            .map(|i| {
                self.state.main_table.items[*i]
                    .index_to_original_node
                    .expect("Node was not given an index")
            })
            .collect();

        // If no items where selected, delete the focused one
        if indeces.is_empty() {
            match self.state.main_table.focused() {
                Some(i) => {
                    indeces = vec![i
                        .index_to_original_node
                        .expect("Node was not given an index")]
                }
                _ => return,
            }
        }

        if self.tree.delete_entries(indeces).is_err() {
            self.set_message("Error deleting entry".to_string());
        }
        self.update_view();
    }
}
