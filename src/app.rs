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
    pub waiting_for_next_phase: bool,
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
            waiting_for_next_phase: false,
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

        // Handle waiting for next phase confirmation
        if self.waiting_for_next_phase {
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(timer) = &mut self.timer {
                        match timer.phase {
                            TimerPhase::Work => timer.start_break(),
                            TimerPhase::Break => timer.start_work(),
                        }
                    }
                    self.waiting_for_next_phase = false;
                    self.show_completion_message = false;
                }
                KeyCode::Char('q') => self.running = false,
                KeyCode::Char('m') | KeyCode::Esc => {
                    self.waiting_for_next_phase = false;
                    self.show_exit_confirm = true;
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
        if self.waiting_for_next_phase {
            return;
        }

        self.show_completion_message = false;

        if let Some(timer) = &mut self.timer {
            let phase_completed = timer.tick();
            if phase_completed {
                match timer.phase {
                    TimerPhase::Work => {
                        self.analytics.record_pomodoro(timer.mode);
                        self.show_completion_message = true;
                    }
                    TimerPhase::Break => {}
                }
                timer.paused = true;
                self.waiting_for_next_phase = true;
            }
        }
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self {
            screen: Screen::ModeSelection,
            running: true,
            selected_mode: 0,
            timer: None,
            analytics: Analytics::default(),
            show_completion_message: false,
            show_exit_confirm: false,
            waiting_for_next_phase: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_initial_state() {
        let app = App::new_for_test();
        assert_eq!(app.screen, Screen::ModeSelection);
        assert!(app.running);
        assert_eq!(app.selected_mode, 0);
        assert!(app.timer.is_none());
    }

    // Mode Selection tests
    #[test]
    fn test_mode_selection_navigate_down() {
        let mut app = App::new_for_test();
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.selected_mode, 1);

        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.selected_mode, 0); // Wraps around
    }

    #[test]
    fn test_mode_selection_navigate_up() {
        let mut app = App::new_for_test();
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.selected_mode, 1); // Wraps to bottom

        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.selected_mode, 0);
    }

    #[test]
    fn test_mode_selection_arrow_keys() {
        let mut app = App::new_for_test();
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.selected_mode, 1);

        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.selected_mode, 0);
    }

    #[test]
    fn test_mode_selection_start_short() {
        let mut app = App::new_for_test();
        app.selected_mode = 0;
        app.handle_key(key(KeyCode::Enter));

        assert_eq!(app.screen, Screen::Timer);
        assert!(app.timer.is_some());
        assert_eq!(app.timer.as_ref().unwrap().mode, PomodoroMode::Short);
    }

    #[test]
    fn test_mode_selection_start_long() {
        let mut app = App::new_for_test();
        app.selected_mode = 1;
        app.handle_key(key(KeyCode::Enter));

        assert_eq!(app.screen, Screen::Timer);
        assert!(app.timer.is_some());
        assert_eq!(app.timer.as_ref().unwrap().mode, PomodoroMode::Long);
    }

    #[test]
    fn test_mode_selection_go_to_analytics() {
        let mut app = App::new_for_test();
        app.handle_key(key(KeyCode::Char('a')));

        assert_eq!(app.screen, Screen::Analytics);
    }

    #[test]
    fn test_mode_selection_quit() {
        let mut app = App::new_for_test();
        app.handle_key(key(KeyCode::Char('q')));

        assert!(!app.running);
    }

    // Timer tests
    #[test]
    fn test_timer_pause() {
        let mut app = App::new_for_test();
        app.timer = Some(Timer::new(PomodoroMode::Short));
        app.screen = Screen::Timer;

        assert!(!app.timer.as_ref().unwrap().paused);
        app.handle_key(key(KeyCode::Char(' ')));
        assert!(app.timer.as_ref().unwrap().paused);
    }

    #[test]
    fn test_timer_reset() {
        let mut app = App::new_for_test();
        app.timer = Some(Timer::new(PomodoroMode::Short));
        app.screen = Screen::Timer;

        app.timer.as_mut().unwrap().remaining = std::time::Duration::from_secs(100);
        app.handle_key(key(KeyCode::Char('r')));

        assert_eq!(
            app.timer.as_ref().unwrap().remaining,
            std::time::Duration::from_secs(25 * 60)
        );
    }

    #[test]
    fn test_timer_skip_work_to_break() {
        let mut app = App::new_for_test();
        app.timer = Some(Timer::new(PomodoroMode::Short));
        app.screen = Screen::Timer;

        app.handle_key(key(KeyCode::Char('s')));

        assert_eq!(app.timer.as_ref().unwrap().phase, TimerPhase::Break);
        assert!(app.show_completion_message);
    }

    #[test]
    fn test_timer_skip_break_to_work() {
        let mut app = App::new_for_test();
        app.timer = Some(Timer::new(PomodoroMode::Short));
        app.timer.as_mut().unwrap().start_break();
        app.screen = Screen::Timer;

        app.handle_key(key(KeyCode::Char('s')));

        assert_eq!(app.timer.as_ref().unwrap().phase, TimerPhase::Work);
        assert!(!app.show_completion_message);
    }

    #[test]
    fn test_timer_exit_shows_confirm() {
        let mut app = App::new_for_test();
        app.timer = Some(Timer::new(PomodoroMode::Short));
        app.screen = Screen::Timer;

        app.handle_key(key(KeyCode::Char('m')));

        assert!(app.show_exit_confirm);
        assert!(app.timer.as_ref().unwrap().paused);
        assert_eq!(app.screen, Screen::Timer); // Still on timer screen
    }

    #[test]
    fn test_timer_exit_confirm_yes() {
        let mut app = App::new_for_test();
        app.timer = Some(Timer::new(PomodoroMode::Short));
        app.screen = Screen::Timer;
        app.show_exit_confirm = true;

        app.handle_key(key(KeyCode::Char('y')));

        assert!(!app.show_exit_confirm);
        assert!(app.timer.is_none());
        assert_eq!(app.screen, Screen::ModeSelection);
    }

    #[test]
    fn test_timer_exit_confirm_no() {
        let mut app = App::new_for_test();
        app.timer = Some(Timer::new(PomodoroMode::Short));
        app.screen = Screen::Timer;
        app.show_exit_confirm = true;

        app.handle_key(key(KeyCode::Char('n')));

        assert!(!app.show_exit_confirm);
        assert!(app.timer.is_some());
        assert_eq!(app.screen, Screen::Timer);
    }

    #[test]
    fn test_timer_quit() {
        let mut app = App::new_for_test();
        app.timer = Some(Timer::new(PomodoroMode::Short));
        app.screen = Screen::Timer;

        app.handle_key(key(KeyCode::Char('q')));

        assert!(!app.running);
    }

    // Analytics tests
    #[test]
    fn test_analytics_back() {
        let mut app = App::new_for_test();
        app.screen = Screen::Analytics;

        app.handle_key(key(KeyCode::Char('b')));

        assert_eq!(app.screen, Screen::ModeSelection);
    }

    #[test]
    fn test_analytics_back_esc() {
        let mut app = App::new_for_test();
        app.screen = Screen::Analytics;

        app.handle_key(key(KeyCode::Esc));

        assert_eq!(app.screen, Screen::ModeSelection);
    }

    #[test]
    fn test_analytics_quit() {
        let mut app = App::new_for_test();
        app.screen = Screen::Analytics;

        app.handle_key(key(KeyCode::Char('q')));

        assert!(!app.running);
    }
}
