use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PomodoroMode {
    Short, // 25 min work, 5 min break
    Long,  // 50 min work, 10 min break
}

impl PomodoroMode {
    pub fn work_duration(&self) -> Duration {
        match self {
            PomodoroMode::Short => Duration::from_secs(25 * 60),
            PomodoroMode::Long => Duration::from_secs(50 * 60),
        }
    }

    pub fn break_duration(&self) -> Duration {
        match self {
            PomodoroMode::Short => Duration::from_secs(5 * 60),
            PomodoroMode::Long => Duration::from_secs(10 * 60),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PomodoroMode::Short => "Short (25/5)",
            PomodoroMode::Long => "Long (50/10)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerPhase {
    Work,
    Break,
}

impl TimerPhase {
    pub fn name(&self) -> &'static str {
        match self {
            TimerPhase::Work => "Work",
            TimerPhase::Break => "Break",
        }
    }
}

#[derive(Debug)]
pub struct Timer {
    pub mode: PomodoroMode,
    pub phase: TimerPhase,
    pub remaining: Duration,
    pub paused: bool,
    last_tick: Instant,
}

impl Timer {
    pub fn new(mode: PomodoroMode) -> Self {
        Self {
            mode,
            phase: TimerPhase::Work,
            remaining: mode.work_duration(),
            paused: false,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.paused {
            self.last_tick = Instant::now();
            return false;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;

        if elapsed >= self.remaining {
            self.remaining = Duration::ZERO;
            true // Phase completed
        } else {
            self.remaining -= elapsed;
            false
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if !self.paused {
            self.last_tick = Instant::now();
        }
    }

    pub fn reset(&mut self) {
        self.remaining = match self.phase {
            TimerPhase::Work => self.mode.work_duration(),
            TimerPhase::Break => self.mode.break_duration(),
        };
        self.paused = false;
        self.last_tick = Instant::now();
    }

    pub fn start_break(&mut self) {
        self.phase = TimerPhase::Break;
        self.remaining = self.mode.break_duration();
        self.paused = false;
        self.last_tick = Instant::now();
    }

    pub fn start_work(&mut self) {
        self.phase = TimerPhase::Work;
        self.remaining = self.mode.work_duration();
        self.paused = false;
        self.last_tick = Instant::now();
    }

    pub fn skip_phase(&mut self) -> bool {
        // Returns true if work phase was skipped (pomodoro completed)
        let was_work = self.phase == TimerPhase::Work;
        match self.phase {
            TimerPhase::Work => self.start_break(),
            TimerPhase::Break => self.start_work(),
        }
        was_work
    }

    pub fn progress(&self) -> f64 {
        let total = match self.phase {
            TimerPhase::Work => self.mode.work_duration(),
            TimerPhase::Break => self.mode.break_duration(),
        };
        1.0 - (self.remaining.as_secs_f64() / total.as_secs_f64())
    }

    pub fn format_remaining(&self) -> String {
        let secs = self.remaining.as_secs();
        let minutes = secs / 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_mode_durations() {
        assert_eq!(
            PomodoroMode::Short.work_duration(),
            Duration::from_secs(25 * 60)
        );
        assert_eq!(
            PomodoroMode::Short.break_duration(),
            Duration::from_secs(5 * 60)
        );
    }

    #[test]
    fn test_long_mode_durations() {
        assert_eq!(
            PomodoroMode::Long.work_duration(),
            Duration::from_secs(50 * 60)
        );
        assert_eq!(
            PomodoroMode::Long.break_duration(),
            Duration::from_secs(10 * 60)
        );
    }

    #[test]
    fn test_mode_names() {
        assert_eq!(PomodoroMode::Short.name(), "Short (25/5)");
        assert_eq!(PomodoroMode::Long.name(), "Long (50/10)");
    }

    #[test]
    fn test_phase_names() {
        assert_eq!(TimerPhase::Work.name(), "Work");
        assert_eq!(TimerPhase::Break.name(), "Break");
    }

    #[test]
    fn test_timer_new() {
        let timer = Timer::new(PomodoroMode::Short);
        assert_eq!(timer.mode, PomodoroMode::Short);
        assert_eq!(timer.phase, TimerPhase::Work);
        assert_eq!(timer.remaining, Duration::from_secs(25 * 60));
        assert!(!timer.paused);
    }

    #[test]
    fn test_toggle_pause() {
        let mut timer = Timer::new(PomodoroMode::Short);
        assert!(!timer.paused);

        timer.toggle_pause();
        assert!(timer.paused);

        timer.toggle_pause();
        assert!(!timer.paused);
    }

    #[test]
    fn test_reset_work_phase() {
        let mut timer = Timer::new(PomodoroMode::Short);
        timer.remaining = Duration::from_secs(100);
        timer.paused = true;

        timer.reset();

        assert_eq!(timer.remaining, Duration::from_secs(25 * 60));
        assert!(!timer.paused);
    }

    #[test]
    fn test_reset_break_phase() {
        let mut timer = Timer::new(PomodoroMode::Short);
        timer.start_break();
        timer.remaining = Duration::from_secs(100);

        timer.reset();

        assert_eq!(timer.remaining, Duration::from_secs(5 * 60));
    }

    #[test]
    fn test_start_break() {
        let mut timer = Timer::new(PomodoroMode::Short);
        timer.start_break();

        assert_eq!(timer.phase, TimerPhase::Break);
        assert_eq!(timer.remaining, Duration::from_secs(5 * 60));
    }

    #[test]
    fn test_start_work() {
        let mut timer = Timer::new(PomodoroMode::Long);
        timer.start_break();
        timer.start_work();

        assert_eq!(timer.phase, TimerPhase::Work);
        assert_eq!(timer.remaining, Duration::from_secs(50 * 60));
    }

    #[test]
    fn test_skip_phase_from_work() {
        let mut timer = Timer::new(PomodoroMode::Short);
        let was_work = timer.skip_phase();

        assert!(was_work);
        assert_eq!(timer.phase, TimerPhase::Break);
    }

    #[test]
    fn test_skip_phase_from_break() {
        let mut timer = Timer::new(PomodoroMode::Short);
        timer.start_break();
        let was_work = timer.skip_phase();

        assert!(!was_work);
        assert_eq!(timer.phase, TimerPhase::Work);
    }

    #[test]
    fn test_progress_at_start() {
        let timer = Timer::new(PomodoroMode::Short);
        assert!((timer.progress() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_progress_halfway() {
        let mut timer = Timer::new(PomodoroMode::Short);
        timer.remaining = Duration::from_secs(12 * 60 + 30); // Half of 25 min
        assert!((timer.progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_progress_at_end() {
        let mut timer = Timer::new(PomodoroMode::Short);
        timer.remaining = Duration::ZERO;
        assert!((timer.progress() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_format_remaining() {
        let mut timer = Timer::new(PomodoroMode::Short);
        assert_eq!(timer.format_remaining(), "25:00");

        timer.remaining = Duration::from_secs(5 * 60 + 30);
        assert_eq!(timer.format_remaining(), "05:30");

        timer.remaining = Duration::from_secs(59);
        assert_eq!(timer.format_remaining(), "00:59");
    }

    #[test]
    fn test_tick_when_paused() {
        let mut timer = Timer::new(PomodoroMode::Short);
        timer.paused = true;
        let original = timer.remaining;

        let completed = timer.tick();

        assert!(!completed);
        assert_eq!(timer.remaining, original);
    }

    #[test]
    fn test_tick_completes_phase() {
        let mut timer = Timer::new(PomodoroMode::Short);
        timer.remaining = Duration::from_millis(1);

        std::thread::sleep(Duration::from_millis(10));
        let completed = timer.tick();

        assert!(completed);
        assert_eq!(timer.remaining, Duration::ZERO);
    }
}
