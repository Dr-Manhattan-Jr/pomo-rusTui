use chrono::{DateTime, Datelike, Local, NaiveDate};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::timer::PomodoroMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroRecord {
    pub timestamp: DateTime<Local>,
    pub mode: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Analytics {
    pub records: Vec<PomodoroRecord>,
}

impl Analytics {
    fn data_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "pomo").map(|dirs| {
            let path = dirs.data_dir().join("rustui");
            fs::create_dir_all(&path).ok();
            path.join("analytics.json")
        })
    }

    pub fn load() -> Self {
        Self::data_path()
            .and_then(|path| fs::read_to_string(&path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::data_path() {
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(&path, content);
            }
        }
    }

    pub fn record_pomodoro(&mut self, mode: PomodoroMode) {
        self.records.push(PomodoroRecord {
            timestamp: Local::now(),
            mode: mode.name().to_string(),
        });
        self.save();
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.save();
    }

    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    pub fn today_count(&self) -> usize {
        let today = Local::now().date_naive();
        self.records
            .iter()
            .filter(|r| r.timestamp.date_naive() == today)
            .count()
    }

    pub fn week_count(&self) -> usize {
        let now = Local::now();
        let today = now.date_naive();
        let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);

        self.records
            .iter()
            .filter(|r| {
                let date = r.timestamp.date_naive();
                date >= week_start && date <= today
            })
            .count()
    }

    pub fn current_streak(&self) -> usize {
        if self.records.is_empty() {
            return 0;
        }

        let mut dates: Vec<NaiveDate> = self
            .records
            .iter()
            .map(|r| r.timestamp.date_naive())
            .collect();
        dates.sort();
        dates.dedup();

        let today = Local::now().date_naive();
        let yesterday = today - chrono::Duration::days(1);

        // Check if there's activity today or yesterday
        if dates.last().map(|&d| d != today && d != yesterday).unwrap_or(true) {
            return 0;
        }

        let mut streak = 0;
        let mut current_date = if dates.contains(&today) { today } else { yesterday };

        for date in dates.iter().rev() {
            if *date == current_date {
                streak += 1;
                current_date -= chrono::Duration::days(1);
            } else if *date < current_date {
                break;
            }
        }

        streak
    }

    pub fn short_mode_count(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.mode.contains("Short"))
            .count()
    }

    pub fn long_mode_count(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.mode.contains("Long"))
            .count()
    }
}
