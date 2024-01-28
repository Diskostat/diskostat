/// Application.
pub mod app;

/// Handlers terminal events and creates render and tick events.
pub mod event_handling;

/// Handles received events.
pub mod disko_event_handling;

/// Widget renderer.
pub mod renderer;

/// Terminal user interface.
pub mod tui;

/// Component logic used for rendering UI widgets.
pub mod components;

/// Struct specifying the color theme of the application.
pub mod color_theme;
