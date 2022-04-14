use std::io;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub tick_rate: Option<Duration>,
    pub exit_key: Key,
}

impl Config {
    pub fn without_ticker() -> Self {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: None,
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Some(Duration::from_millis(250)),
        }
    }
}

impl Events {
    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for key in stdin.keys().flatten() {
                    if let Err(err) = tx.send(Event::Input(key)) {
                        eprintln!("{}", err);
                        return;
                    }
                    if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                        return;
                    }
                }
            })
        };

        let tick_handle = config.tick_rate.map(|d| {
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(d);
            })
        });
        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }

    pub fn wait(self) {
        let _ = self.input_handle.join();
        if let Some(h) = self.tick_handle {
            let _ = h.join();
        }
    }
}
impl Default for Events {
    fn default() -> Events {
        Self::with_config(Config::default())
    }
}
