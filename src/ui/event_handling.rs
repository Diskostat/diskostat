use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Initialize the terminal application.
    Init,
    /// Render the terminal application.
    Render,
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    /// The rate at which [`Event::Tick`] events should be sent.
    tick_rate: f64,
    /// The rate at which [`Event::Render`] events should be sent.
    render_rate: f64,
    /// Event handler threads.
    handlers: Vec<thread::JoinHandle<()>>,
    /// Whether the event handler should cancel.
    should_cancel: Arc<AtomicBool>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(
        tick_rate: f64,
        render_rate: f64,
        sender: mpsc::Sender<Event>,
        receiver: mpsc::Receiver<Event>,
    ) -> Self {
        Self {
            sender,
            receiver,
            tick_rate,
            render_rate,
            handlers: Vec::new(),
            should_cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Starts the processing of events.
    pub fn start(&mut self) -> Result<()> {
        self.sender
            .send(Event::Init)
            .expect("Failed to send init event");

        let tick_delay = Duration::try_from_secs_f64(1.0 / self.tick_rate)?;

        let tick_sender = self.sender.clone();
        let cancel_tick_handler = self.should_cancel.clone();
        let tick_handler =
            thread::spawn(move || Self::handle_tick(tick_delay, tick_sender, cancel_tick_handler));
        self.handlers.push(tick_handler);

        let render_sender = self.sender.clone();
        let cancel_render_handler = self.should_cancel.clone();
        let render_delay = Duration::try_from_secs_f64(1.0 / self.render_rate)?;
        let render_handler = thread::spawn(move || {
            Self::handle_render(render_delay, render_sender, cancel_render_handler)
        });
        self.handlers.push(render_handler);

        Ok(())
    }

    fn handle_tick(
        tick_delay: Duration,
        tick_sender: mpsc::Sender<Event>,
        cancel_tick_handler: Arc<AtomicBool>,
    ) {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_delay
                .checked_sub(last_tick.elapsed())
                .unwrap_or(tick_delay);

            if crossterm::event::poll(timeout).expect("Unable to poll for event") {
                let event = Self::read_crossterm_event();

                if let Some(event) = event {
                    tick_sender
                        .send(event)
                        .expect("Failed to send crossterm event.");
                }
            }

            if last_tick.elapsed() >= tick_delay {
                tick_sender
                    .send(Event::Tick)
                    .expect("Failed to send tick event.");
                last_tick = Instant::now();
            }

            if cancel_tick_handler.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    fn read_crossterm_event() -> Option<Event> {
        match crossterm::event::read().expect("Unable to read event.") {
            CrosstermEvent::Key(e) => {
                if e.kind == KeyEventKind::Press {
                    Some(Event::Key(e))
                } else {
                    None // ignore KeyEventKind::Release on windows
                }
            }
            CrosstermEvent::Mouse(e) => Some(Event::Mouse(e)),
            CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
            _ => None,
        }
    }

    fn handle_render(
        render_delay: Duration,
        render_sender: mpsc::Sender<Event>,
        cancel_render_handler: Arc<AtomicBool>,
    ) {
        loop {
            thread::sleep(render_delay);
            render_sender
                .send(Event::Render)
                .expect("Failed to send render event.");

            if cancel_render_handler.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    /// Stops the processing of events.
    pub fn stop(&mut self) -> Result<()> {
        self.should_cancel.store(true, Ordering::SeqCst);
        while let Some(handler) = self.handlers.pop() {
            handler
                .join()
                .expect("Failed to join event handler thread.");
        }
        Ok(())
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}