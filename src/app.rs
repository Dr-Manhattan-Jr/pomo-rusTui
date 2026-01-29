use crossterm::event::{KeyCode, KeyEvent};

use crate::analytics::Analytics;
use crate::timer::{PomodoroMode, Timer, TimerPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    ModeSelection,
    Timer,
    Analytics,
}

pub struct App {
    pub screen: Screen,
    pub running: bool,
    pub selected_mode: usize,
    pub timer: Option<Timer>,
    pub analytics: Analytics,
    pub show_completion_message: bool,
    pub show_exit_confirm: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::ModeSelection,
            running: true,
            selected_mode: 0,
            timer: None,
            analytics: Analytics::load(),
            show_completion_message: false,
            show_exit_confirm: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.screen {
            Screen::ModeSelection => self.handle_mode_selection_key(key),
            Screen::Timer => self.handle_timer_key(key),
            Screen::Analytics => self.handle_analytics_key(key),
        }
    }

    fn handle_mode_selection_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_mode = (self.selected_mode + 1) % 2;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_mode = if self.selected_mode == 0 { 1 } else { 0 };
            }
            KeyCode::Enter => {
                let mode = if self.selected_mode == 0 {
                    PomodoroMode::Short
                } else {
                    PomodoroMode::Long
                };
                self.timer = Some(Timer::new(mode));
                self.screen = Screen::Timer;
            }
            KeyCode::Char('a') => {
                self.screen = Screen::Analytics;
            }
            _ => {}
        }
    }

    fn handle_timer_key(&mut self, key: KeyEvent) {
        // Handle exit confirmation dialog
        if self.show_exit_confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    self.show_exit_confirm = false;
                    self.timer = None;
                    self.screen = Screen::ModeSelection;
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.show_exit_confirm = false;
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char(' ') => {
                if let Some(timer) = &mut self.timer {
                    timer.toggle_pause();
                }
            }
            KeyCode::Char('r') => {
                if let Some(timer) = &mut self.timer {
                    timer.reset();
                }
            }
            KeyCode::Char('s') => {
                if let Some(timer) = &mut self.timer {
                    let was_work = timer.skip_phase();
                    if was_work {
                        self.analytics.record_pomodoro(timer.mode);
                        self.show_completion_message = true;
                    }
                }
            }
            KeyCode::Char('m') | KeyCode::Esc => {
                // Pause timer and show confirmation
                if let Some(timer) = &mut self.timer {
                    timer.paused = true;
                }
                self.show_exit_confirm = true;
            }
            _ => {}
        }
    }

    fn handle_analytics_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('b') | KeyCode::Esc => {
                self.screen = Screen::ModeSelection;
            }
            KeyCode::Char('c') => {
                self.analytics.clear();
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.show_completion_message = false;

        if let Some(timer) = &mut self.timer {
            let phase_completed = timer.tick();
            if phase_completed {
                match timer.phase {
                    TimerPhase::Work => {
                        self.analytics.record_pomodoro(timer.mode);
                        self.show_completion_message = true;
                        timer.start_break();
                    }
                    TimerPhase::Break => {
                        timer.start_work();
                    }
                }
            }
        }
    }
}
