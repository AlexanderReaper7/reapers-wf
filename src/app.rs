use std::sync::Arc;

use crate::{config::Config, fissure_watcher, models::Fissure};
use ratatui::{text::Text, widgets::*};
use tokio::sync::{RwLock, mpsc};

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub struct StatefulList<'a> {
    pub state: ListState,
    pub list: Vec<Text<'a>>,
}

impl<'a> StatefulList<'a> {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub struct App<'a> {
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub console_log: StatefulList<'a>,
    pub current_cmd: String,
    pub config: Arc<RwLock<Config>>,
    pub fissure_watcher: fissure_watcher::FissureWatcher,
}

impl<'a> App<'a> {
    pub async fn new() -> App<'a> {
        // init console log
        let mut console_log = StatefulList {
            state: ListState::default(),
            list: vec![Text::raw("Starting Reaper's Warframe Tools")],
        };
        // load config
        let (config, text) = App::load_config().await;
        let config = Arc::new(RwLock::new(config));
        console_log.list.push(text);
        // start fissure watcher
        let fissure_watcher = fissure_watcher::FissureWatcher::new(config.clone());
        App {
            should_quit: false,
            tabs: TabsState::new(vec!["Console", "Fissures", "Settings"]),
            console_log,
            current_cmd: String::new(),
            config,
            fissure_watcher,
        }
    }

    fn exec_cmd(&mut self) {
        match self.current_cmd.as_str() {
            "quit" => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub(crate) fn on_up(&mut self) {
        match self.tabs.index {
            0 => {
                self.console_log.previous();
            }
            1 => {
                self.fissure_watcher.previous();
            }
            _ => {}
        };
    }

    pub(crate) fn on_down(&mut self) {
        match self.tabs.index {
            0 => {
                self.console_log.next();
            }
            1 => {
                self.fissure_watcher.next();
            }
            _ => {}
        };
    }

    pub(crate) fn on_right(&mut self) {
        self.tabs.next();
    }

    pub(crate) fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub(crate) fn on_key(&mut self, c: char) {
        match self.tabs.index {
            0 => {
                self.current_cmd.push(c);
            }
            _ => {}
        };
    }

    /// The primary tick function for the application.
    pub(crate) fn update(&mut self) {
        // Go through all the events sent from the worker threads
        // Fissure watcher
        while let Ok(event) = self.fissure_watcher.fissure_rx.try_recv() {
            let time_format: Vec<time::format_description::FormatItem<'_>> = time::format_description::parse(
                "[hour]:[minute]:[second]").unwrap();
            let now = time::OffsetDateTime::now_utc();
            let time_stamp = now.format(&time_format).unwrap();
            match event {
                fissure_watcher::Event::Fissures{fissures, filtered_fissures, new_count} => {
                    self.fissure_watcher.update_fissures(fissures, filtered_fissures);
                    self.console_log.list.push(Text::raw(format!("[{}] {} new fissures", time_stamp, new_count)));
                }
                fissure_watcher::Event::Err(e) => {
                    self.console_log.list.push(Text::raw(format!("[{}] Error: {}", time_stamp, e)));
                }
                fissure_watcher::Event::NoNewFissures => {
                    self.console_log.list.push(Text::raw(format!("[{}] No new fissures", time_stamp)));
                },
                
            }
        }
        // when the fissures or the filters change we need to update the filtered fissures

    }

    async fn load_config() -> (Config, Text<'a>) {
        let config = Config::load().await;
        match config {
            Ok(config) => (config, Text::raw("Loaded config file.")),
            Err(e) => match e.downcast_ref::<toml::de::Error>() {
                Some(toml::de::Error { .. }) => {
                    (Config::default(), Text::raw(
                            "Error parsing config file, if you have edited it, please fix it, otherwise delete it and restart the program.\nContinuing with default config.",
                        ))
                }
                _ => {
                    Config::create_default_file().await.unwrap();
                    (Config::default(), Text::raw("Error loading config file, creating and using default config file."))
                },
            },
        }
    }

    pub(crate) fn on_backspace(&mut self) {
        match self.tabs.index {
            0 => {
                self.current_cmd.pop();
            }
            _ => {}
        };
    }

    pub(crate) fn on_esc(&mut self) {
        self.should_quit = true;
    }

    pub(crate) fn on_enter(&mut self) {
        match self.tabs.index {
            0 => {
                self.exec_cmd();
                self.current_cmd.clear();
            }
            _ => {}
        };
    }
}
