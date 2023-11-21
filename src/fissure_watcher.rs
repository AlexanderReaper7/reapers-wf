use crate::api;
use crate::config::Config;
use crate::models::Fissure;
use notify_rust::Notification;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Row, Table, TableState};
use ratatui::Frame;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub struct FissureWatcher {
    fissures: Vec<Fissure>,
    filtered_fissures: Vec<Fissure>,
    pub fissure_rx: mpsc::Receiver<Event>,
    pub fissure_handle: tokio::task::JoinHandle<()>,
    table_rows: Vec<Vec<String>>,
    table_state: TableState,
}
impl FissureWatcher {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        let (fissure_tx, fissure_rx) = mpsc::channel::<Event>(20);
        let fissure_handle = run(Arc::clone(&config), fissure_tx);
        Self {
            fissures: Vec::new(),
            filtered_fissures: Vec::new(),
            fissure_rx,
            fissure_handle,
            table_rows: Vec::new(),
            table_state: TableState::default(),
        }
    }

    pub fn update_fissures(&mut self, fissures: Vec<Fissure>, filtered_fissures: Vec<Fissure>) {
        self.fissures = fissures;
        self.update_filtered_fissures(filtered_fissures);
    }

    pub fn update_filtered_fissures(&mut self, filtered_fissures: Vec<Fissure>) {
        self.filtered_fissures = filtered_fissures;
        self.table_rows = self
            .filtered_fissures
            .iter()
            .map(|fissure| fissure.table_string())
            .collect::<Vec<Vec<String>>>();
    }

    pub fn next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.filtered_fissures.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_fissures.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let header = Fissure::table_headers();
        let widths = crate::ui::calculate_table_widths(&header, &self.table_rows);
        let fissure_table = Table::new(self.table_rows.iter().cloned().map(|row| Row::new(row)))
            .header(Row::new(header))
            .widths(&widths)
            .column_spacing(3)
            .highlight_style(Style::default().bold());
        f.render_stateful_widget(fissure_table, area, &mut self.table_state);
    }
}

pub enum Event {
    Fissures {
        fissures: Vec<Fissure>,
        filtered_fissures: Vec<Fissure>,
        new_count: usize,
    },
    NoNewFissures,
    Err(String),
}

pub fn run(config: Arc<RwLock<Config>>, tx: Sender<Event>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let sender = tx;
        let mut interval =
            tokio::time::interval(Duration::from_secs(config.read().await.refresh_rate));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut fissures = Vec::new();
        loop {
            interval.tick().await;
            // check for new fissures
            if let Ok((new_count, removed_count)) = update_fissures(&mut fissures).await {
                if new_count > 0 || removed_count > 0 {
                    if new_count > 0 {
                        // apply filters to new fissures
                        let new_fissures =
                            &fissures[(fissures.len() - new_count)..(fissures.len())].to_vec();
                        let filtered_fissures = config.read().await.apply_filters(new_fissures); // WARN: unnecessary clone?
                                                                                                 // send notification
                        if filtered_fissures.len() > 0 {
                            spawn_notifications(
                                &filtered_fissures,
                                config.read().await.time_before_expiry_notification,
                            )
                            .await;
                        }
                    }
                    let filtered_fissures = config.read().await.apply_filters_cloned(&fissures);
                    sender
                        .send(Event::Fissures {
                            fissures: fissures.clone(),
                            filtered_fissures,
                            new_count,
                        })
                        .await
                        .unwrap();
                } else {
                    sender.send(Event::NoNewFissures).await.unwrap();
                }
            } else {
                sender
                    .send(Event::Err("Failed to fetch fissures".to_string()))
                    .await
                    .unwrap();
            }
        }
    })
}

/// Consumes a vector of Fissures with the current Fissures, returning None if nothing changed or an updated vector and a count of the new Fissures if something did change
pub async fn update_fissures(old: &mut Vec<Fissure>) -> reqwest::Result<(usize, usize)> {
    let current = api::get_fissures().await?;
    // remove expired fissures
    let expired: Vec<usize> = old
        .iter()
        .enumerate()
        .filter(|(_, fissure)| current.iter().find(|f| f.id == fissure.id).is_none())
        .map(|(i, _)| i)
        .collect();
    let mut removed_count = 0;
    for i in expired.into_iter().rev() {
        old.remove(i);
        removed_count += 1;
    }
    // add new fissures
    let mut new_count = 0;
    for fissure in current {
        if old
            .iter()
            .find(|old_fissure| old_fissure.id == fissure.id)
            .is_none()
        {
            old.push(fissure);
            new_count += 1;
        }
    }
    Ok((new_count, removed_count))
}

/// Runs the fissure watcher, returning a Vec of the filtered Fissures and a count of how many are new and
/// Sends a notification of any new Fissures and enqueues a notification for each Fissure's expiry once there are the configured amount of seconds left.
// pub async fn update_filter_notify<'a>(
//     config: &Config,
//     fissures: &'a mut Vec<Fissure>,
// ) -> Result<(Option<Vec<&'a Fissure>>, usize), Box<dyn Error>> {
//     let (filtered, new_count) = update_and_filter_fissures(config, fissures).await?;
//     if let Some(filtered_fissures) = filtered {
//         spawn_notifications(&filtered_fissures, config.time_before_expiry_notification).await;
//         Ok((Some(filtered_fissures), new_count))
//     } else {
//         Ok((None, new_count))
//     }
// }

/// Updates the given vector of Fissures with the current Fissures, returning a Vec of the filtered Fissures and a count of how many are new
// pub async fn update_and_filter_fissures<'a>(
//     config: &Config,
//     fissures: &'a mut Vec<Fissure>,
// ) -> Result<(Option<Vec<&'a Fissure>>, usize), Box<dyn Error>> {
//     let new_count = update_fissures(fissures).await?;
//     if new_count == 0 {
//         Ok((None, 0))
//     } else {
//         let filtered_fissures = config.apply_filters(fissures);
//         if filtered_fissures.len() == 0 {
//             Ok((None, new_count))
//         } else {
//             Ok((Some(filtered_fissures), new_count))
//         }
//     }
// }

/// Sends a notification with the details of each Fissure in the given vector, and enqueues a notification for each Fissure's expiry once there are `time_before_expiry_notification` seconds left.
pub async fn spawn_notifications(fissures: &Vec<&Fissure>, time_before_expiry_notification: u64) {
    // send notification
    Notification::new()
        .summary("New Fissures")
        .body(
            fissures
                .iter()
                .map(|fissure| fissure.to_string())
                .collect::<Vec<String>>()
                .join("\n")
                .as_str(),
        )
        .show()
        .unwrap();
    // enqueue notification for expiry
    for fissure in fissures {
        spawn_expiry_notification(fissure, time_before_expiry_notification);
    }
}

/// Spawns a new tokio task that sends a notification of the given Fissure's expiry once there are `time_before_expiry_notification` seconds left before it expires.
fn spawn_expiry_notification(fissure: &Fissure, time_before_expiry_notification: u64) {
    let expiry = fissure.expiry - Duration::from_secs(time_before_expiry_notification);
    let now = OffsetDateTime::now_utc();
    if expiry > now {
        let duration = expiry - now;
        let sleep_duration = Duration::from_secs_f64(duration.as_seconds_f64());
        let fissure_str = fissure.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(sleep_duration).await;
            Notification::new()
                .summary(
                    format!(
                        "Fissure is Expiring In {} Seconds",
                        time_before_expiry_notification
                    )
                    .as_str(),
                )
                .body(fissure_str.as_str())
                .show()
                .unwrap();
        });
    }
}
