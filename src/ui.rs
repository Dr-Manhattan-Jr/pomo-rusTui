use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
};

use crate::app::{App, Screen};
use crate::timer::TimerPhase;

// Color palette
const PRIMARY: Color = Color::Rgb(255, 107, 107); // #FF6B6B - Tomato red
const SECONDARY: Color = Color::Rgb(78, 205, 196); // #4ECDC4 - Turquoise
const ACCENT: Color = Color::Rgb(255, 230, 109); // #FFE66D - Yellow
const WORK_COLOR: Color = Color::Rgb(249, 115, 22); // #F97316 - Orange
const BREAK_COLOR: Color = Color::Rgb(34, 197, 94); // #22C55E - Green
const BG_DARK: Color = Color::Rgb(30, 30, 46); // #1E1E2E - Dark

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Background
    let bg_block = Block::default().style(Style::default().bg(BG_DARK));
    frame.render_widget(bg_block, area);

    match app.screen {
        Screen::ModeSelection => draw_mode_selection(frame, app, area),
        Screen::Timer => draw_timer(frame, app, area),
        Screen::Analytics => draw_analytics(frame, app, area),
    }
}

fn draw_mode_selection(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "  POMODORO  ",
            Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(Span::styled(
            "Select a mode",
            Style::default().fg(Color::Gray),
        )),
    ])
    .alignment(Alignment::Center)
    .block(Block::default());
    frame.render_widget(title, chunks[0]);

    // Mode options
    let modes = ["  Short (25/5)  ", "  Long (50/10)  "];
    let mode_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(centered_rect(40, 6, chunks[2]));

    for (i, mode) in modes.iter().enumerate() {
        let style = if i == app.selected_mode {
            Style::default()
                .fg(BG_DARK)
                .bg(if i == 0 { WORK_COLOR } else { SECONDARY })
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let indicator = if i == app.selected_mode { " " } else { "  " };
        let text = format!("{}{}", indicator, mode);
        let option = Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if i == app.selected_mode {
                        if i == 0 { WORK_COLOR } else { SECONDARY }
                    } else {
                        Color::DarkGray
                    })),
            );
        frame.render_widget(option, mode_chunks[i]);
    }

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("j/k", Style::default().fg(ACCENT)),
        Span::raw(" navigate  "),
        Span::styled("Enter", Style::default().fg(ACCENT)),
        Span::raw(" confirm  "),
        Span::styled("a", Style::default().fg(ACCENT)),
        Span::raw(" analytics  "),
        Span::styled("q", Style::default().fg(ACCENT)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Gray));
    frame.render_widget(help, chunks[3]);
}

fn draw_timer(frame: &mut Frame, app: &App, area: Rect) {
    let timer = match &app.timer {
        Some(t) => t,
        None => return,
    };

    let phase_color = match timer.phase {
        TimerPhase::Work => WORK_COLOR,
        TimerPhase::Break => BREAK_COLOR,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Mode and phase
    let status = Paragraph::new(vec![
        Line::from(Span::styled(
            timer.mode.name(),
            Style::default().fg(SECONDARY),
        )),
        Line::from(Span::styled(
            timer.phase.name(),
            Style::default()
                .fg(phase_color)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(status, chunks[0]);

    // Pause indicator
    if timer.paused {
        let paused = Paragraph::new(Span::styled(
            " PAUSED ",
            Style::default()
                .fg(ACCENT)
                .add_modifier(Modifier::SLOW_BLINK),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(paused, chunks[1]);
    }

    // Timer display
    let time_display = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            timer.format_remaining(),
            Style::default()
                .fg(phase_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ])
    .alignment(Alignment::Center)
    .style(Style::default())
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(phase_color))
            .title(Span::styled(
                format!(" {} ", timer.phase.name()),
                Style::default().fg(phase_color),
            )),
    );

    let timer_area = centered_rect(30, 7, chunks[2]);
    frame.render_widget(time_display, timer_area);

    // Progress bar
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(Style::default().fg(phase_color).bg(Color::DarkGray))
        .percent((timer.progress() * 100.0) as u16)
        .label(Span::styled(
            format!("{:.0}%", timer.progress() * 100.0),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    let gauge_area = centered_rect(60, 3, chunks[3]);
    frame.render_widget(gauge, gauge_area);

    // Completion message
    if app.show_completion_message {
        let msg = Paragraph::new(Span::styled(
            " Pomodoro completed! ",
            Style::default()
                .fg(BG_DARK)
                .bg(BREAK_COLOR)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(msg, chunks[4]);
    }

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Space", Style::default().fg(ACCENT)),
        Span::raw(" pause  "),
        Span::styled("r", Style::default().fg(ACCENT)),
        Span::raw(" reset  "),
        Span::styled("s", Style::default().fg(ACCENT)),
        Span::raw(" skip  "),
        Span::styled("m", Style::default().fg(ACCENT)),
        Span::raw(" menu  "),
        Span::styled("q", Style::default().fg(ACCENT)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Gray));
    frame.render_widget(help, chunks[5]);

    // Exit confirmation dialog
    if app.show_exit_confirm {
        draw_exit_confirm(frame, area);
    }
}

fn draw_exit_confirm(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 7, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    let popup = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Exit to menu?",
            Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Timer will be stopped and progress lost.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("y/Enter", Style::default().fg(ACCENT)),
            Span::raw(" confirm  "),
            Span::styled("n/Esc", Style::default().fg(ACCENT)),
            Span::raw(" cancel"),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PRIMARY))
            .style(Style::default().bg(BG_DARK)),
    );

    frame.render_widget(popup, popup_area);
}

fn draw_analytics(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Span::styled(
        "  ANALYTICS  ",
        Style::default().fg(SECONDARY).add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Stats
    let stats_area = centered_rect(50, 12, chunks[1]);
    let stats_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(stats_area);

    let stats = [
        ("Today", app.analytics.today_count(), WORK_COLOR),
        ("This week", app.analytics.week_count(), SECONDARY),
        ("Total", app.analytics.total_count(), PRIMARY),
        ("Current streak", app.analytics.current_streak(), ACCENT),
        ("Short mode", app.analytics.short_mode_count(), WORK_COLOR),
        ("Long mode", app.analytics.long_mode_count(), SECONDARY),
    ];

    for (i, (label, count, color)) in stats.iter().enumerate() {
        let stat = Paragraph::new(Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", count),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ),
            if *label == "Current streak" {
                Span::styled(" days", Style::default().fg(Color::DarkGray))
            } else {
                Span::styled(" pomodoros", Style::default().fg(Color::DarkGray))
            },
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(stat, stats_chunks[i]);
    }

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("b/Esc", Style::default().fg(ACCENT)),
        Span::raw(" back  "),
        Span::styled("c", Style::default().fg(ACCENT)),
        Span::raw(" clear data  "),
        Span::styled("q", Style::default().fg(ACCENT)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Gray));
    frame.render_widget(help, chunks[2]);
}

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
