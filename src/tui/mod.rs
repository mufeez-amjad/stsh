use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;

use crossterm::{
    cursor,
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use tokio::{
    sync::mpsc,
    task::{JoinHandle as TokioJoinHandle},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
#[allow(unused)]
pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct Tui {
    /// The terminal instance.
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
    /// The event handler.
    pub event_handler: EventHandler,
    /// The frame rate.
    pub frame_rate: f64,
    /// The tick rate.
    pub tick_rate: f64,
    /// Whether to enable mouse capture.
    pub mouse: bool,
    /// Whether to enable bracketed paste.
    pub paste: bool,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let tick_rate = 4.0;
        let frame_rate = 60.0;

        let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr()))?;
        let event_handler = EventHandler::new();
        let mouse = false;
        let paste = false;

        Ok(Self {
            terminal,
            event_handler,
            frame_rate,
            tick_rate,
            mouse,
            paste,
        })
    }

    pub fn tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    pub fn paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }

    pub fn start(&mut self) {
        self.event_handler.start(self.tick_rate, self.frame_rate);
    }

    pub fn stop(&self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.event_handler.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.event_handler.abort();
            }
            if counter > 100 {
                // log::error!("Failed to abort task in 100 milliseconds for unknown reason");
                break;
            }
        }
        Ok(())
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)?;
        if self.mouse {
            crossterm::execute!(std::io::stderr(), EnableMouseCapture)?;
        }
        if self.paste {
            crossterm::execute!(std::io::stderr(), EnableBracketedPaste)?;
        }
        self.start();
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            if self.paste {
                crossterm::execute!(std::io::stderr(), DisableBracketedPaste)?;
            }
            if self.mouse {
                crossterm::execute!(std::io::stderr(), DisableMouseCapture)?;
            }
            crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.event_handler.cancel();
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_handler.next().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<std::io::Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}

#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
    cancellation_token: CancellationToken,
    task: Option<TokioJoinHandle<()>>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();

        Self {
            sender,
            receiver,
            cancellation_token,
            task: None,
        }
    }

    pub fn start(&mut self, tick_rate: f64, frame_rate: f64) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / frame_rate);

        let _sender = self.sender.clone();

        let cancellation_token = self.cancellation_token.clone();

        self.task = Some(tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);

            // Send an init event to the receiver
            _sender.send(Event::Init).unwrap();

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();

                let mut signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT]).unwrap();

                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    CrosstermEvent::Key(key) => {
                                        if key.kind == KeyEventKind::Press {
                                            _sender.send(Event::Key(key)).unwrap();
                                        }
                                    },
                                    CrosstermEvent::Mouse(mouse) => {
                                        _sender.send(Event::Mouse(mouse)).unwrap();
                                    },
                                    CrosstermEvent::Resize(x, y) => {
                                        _sender.send(Event::Resize(x, y)).unwrap();
                                    },
                                    CrosstermEvent::FocusLost => {
                                        _sender.send(Event::FocusLost).unwrap();
                                    },
                                    CrosstermEvent::FocusGained => {
                                        _sender.send(Event::FocusGained).unwrap();
                                    },
                                    CrosstermEvent::Paste(s) => {
                                        _sender.send(Event::Paste(s)).unwrap();
                                    },
                                }
                            }
                            Some(Err(_)) => {
                                _sender.send(Event::Error).unwrap();
                            }
                            None => {},
                        }
                    },
                    _ = tick_delay => {
                        _sender.send(Event::Tick).unwrap();
                    },
                    _ = render_delay => {
                        _sender.send(Event::Render).unwrap();
                    },
                    maybe_signal = signals.next() => {
                        match maybe_signal {
                            Some(SIGTERM) | Some(SIGINT) | Some(SIGQUIT) | Some(SIGHUP) => {
                                _sender.send(Event::Quit).unwrap();
                            },
                            _ => unreachable!(),
                        }
                    },
                }
            }
        }));
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn is_finished(&self) -> bool {
        match self.task {
            Some(ref task) => task.is_finished(),
            None => true,
        }
    }

    pub fn abort(&self) {
        if let Some(ref task) = self.task {
            task.abort();
        }
    }
}
