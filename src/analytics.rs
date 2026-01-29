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
        if let Some(path) = Self::data_path()
            && let Ok(content) = serde_json::to_string_pretty(self)
        {
            let _ = fs::write(&path, content);
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

    #[cfg(test)]
    pub fn add_record_with_timestamp(&mut self, timestamp: DateTime<Local>, mode: PomodoroMode) {
        self.records.push(PomodoroRecord {
            timestamp,
            mode: mode.name().to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_analytics() -> Analytics {
        Analytics::default()
    }

    #[test]
    fn test_empty_analytics() {
        let analytics = create_test_analytics();
        assert_eq!(analytics.total_count(), 0);
        assert_eq!(analytics.today_count(), 0);
        assert_eq!(analytics.week_count(), 0);
        assert_eq!(analytics.current_streak(), 0);
    }

    #[test]
    fn test_total_count() {
        let mut analytics = create_test_analytics();
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Short);
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Long);
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Short);

        assert_eq!(analytics.total_count(), 3);
    }

    #[test]
    fn test_today_count() {
        let mut analytics = create_test_analytics();
        let today = Local::now();
        let yesterday = today - chrono::Duration::days(1);

        analytics.add_record_with_timestamp(today, PomodoroMode::Short);
        analytics.add_record_with_timestamp(today, PomodoroMode::Short);
        analytics.add_record_with_timestamp(yesterday, PomodoroMode::Short);

        assert_eq!(analytics.today_count(), 2);
    }

    #[test]
    fn test_short_mode_count() {
        let mut analytics = create_test_analytics();
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Short);
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Long);
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Short);

        assert_eq!(analytics.short_mode_count(), 2);
        assert_eq!(analytics.long_mode_count(), 1);
    }

    #[test]
    fn test_clear() {
        let mut analytics = create_test_analytics();
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Short);
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Long);

        analytics.records.clear(); // Use clear without save for tests

        assert_eq!(analytics.total_count(), 0);
    }

    #[test]
    fn test_streak_empty() {
        let analytics = create_test_analytics();
        assert_eq!(analytics.current_streak(), 0);
    }

    #[test]
    fn test_streak_today_only() {
        let mut analytics = create_test_analytics();
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Short);

        assert_eq!(analytics.current_streak(), 1);
    }

    #[test]
    fn test_streak_consecutive_days() {
        let mut analytics = create_test_analytics();
        let today = Local::now();

        for i in 0..5 {
            let date = today - chrono::Duration::days(i);
            analytics.add_record_with_timestamp(date, PomodoroMode::Short);
        }

        assert_eq!(analytics.current_streak(), 5);
    }

    #[test]
    fn test_streak_broken() {
        let mut analytics = create_test_analytics();
        let today = Local::now();

        // Today and yesterday
        analytics.add_record_with_timestamp(today, PomodoroMode::Short);
        analytics.add_record_with_timestamp(today - chrono::Duration::days(1), PomodoroMode::Short);
        // Skip a day, then add one more
        analytics.add_record_with_timestamp(today - chrono::Duration::days(3), PomodoroMode::Short);

        assert_eq!(analytics.current_streak(), 2);
    }

    #[test]
    fn test_streak_no_activity_today_but_yesterday() {
        let mut analytics = create_test_analytics();
        let yesterday = Local::now() - chrono::Duration::days(1);

        analytics.add_record_with_timestamp(yesterday, PomodoroMode::Short);
        analytics.add_record_with_timestamp(yesterday - chrono::Duration::days(1), PomodoroMode::Short);

        assert_eq!(analytics.current_streak(), 2);
    }

    #[test]
    fn test_streak_old_activity_only() {
        let mut analytics = create_test_analytics();
        let old_date = Local::now() - chrono::Duration::days(10);

        analytics.add_record_with_timestamp(old_date, PomodoroMode::Short);

        assert_eq!(analytics.current_streak(), 0);
    }

    #[test]
    fn test_week_count() {
        let mut analytics = create_test_analytics();
        let today = Local::now();

        // Add records for this week
        analytics.add_record_with_timestamp(today, PomodoroMode::Short);
        analytics.add_record_with_timestamp(today - chrono::Duration::days(1), PomodoroMode::Short);

        // Add record from last week
        analytics.add_record_with_timestamp(today - chrono::Duration::days(10), PomodoroMode::Short);

        assert!(analytics.week_count() >= 2);
    }

    #[test]
    fn test_serialization() {
        let mut analytics = create_test_analytics();
        analytics.add_record_with_timestamp(Local::now(), PomodoroMode::Short);

        let json = serde_json::to_string(&analytics).unwrap();
        let loaded: Analytics = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.total_count(), 1);
    }
}
