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
