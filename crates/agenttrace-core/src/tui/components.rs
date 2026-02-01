//! Reusable TUI components
//!
//! This module contains reusable widget components for the TUI.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// A metric card widget showing a single value with a label
pub struct MetricCard<'a> {
    title: &'a str,
    value: String,
    color: Color,
    trend: Option<Trend>,
}

/// Trend indicator
pub enum Trend {
    Up(f64),
    Down(f64),
    Flat,
}

impl<'a> MetricCard<'a> {
    pub fn new(title: &'a str, value: impl Into<String>) -> Self {
        Self {
            title,
            value: value.into(),
            color: Color::White,
            trend: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn trend(mut self, trend: Trend) -> Self {
        self.trend = Some(trend);
        self
    }

    pub fn render(self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let trend_indicator = match self.trend {
            Some(Trend::Up(pct)) => Span::styled(
                format!(" ↑{:.1}%", pct),
                Style::default().fg(Color::Green),
            ),
            Some(Trend::Down(pct)) => Span::styled(
                format!(" ↓{:.1}%", pct),
                Style::default().fg(Color::Red),
            ),
            Some(Trend::Flat) => Span::styled(" →", Style::default().fg(Color::DarkGray)),
            None => Span::raw(""),
        };

        let content = Line::from(vec![
            Span::styled(&self.value, Style::default().fg(self.color)),
            trend_indicator,
        ]);

        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

/// Progress bar with percentage
pub struct ProgressBar {
    value: f64,
    max: f64,
    color: Color,
    show_percentage: bool,
}

impl ProgressBar {
    pub fn new(value: f64, max: f64) -> Self {
        Self {
            value,
            max,
            color: Color::Cyan,
            show_percentage: true,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render_inline(&self, width: usize) -> String {
        let pct = (self.value / self.max).clamp(0.0, 1.0);
        let filled = (pct * width as f64) as usize;
        let empty = width - filled;

        format!(
            "{}{}",
            "█".repeat(filled),
            "░".repeat(empty)
        )
    }
}

/// Status indicator (colored dot with label)
pub struct StatusIndicator<'a> {
    label: &'a str,
    status: Status,
}

pub enum Status {
    Ok,
    Warning,
    Error,
    Unknown,
}

impl<'a> StatusIndicator<'a> {
    pub fn new(label: &'a str, status: Status) -> Self {
        Self { label, status }
    }

    pub fn to_span(&self) -> Span<'a> {
        let (symbol, color) = match self.status {
            Status::Ok => ("●", Color::Green),
            Status::Warning => ("●", Color::Yellow),
            Status::Error => ("●", Color::Red),
            Status::Unknown => ("○", Color::DarkGray),
        };

        Span::styled(
            format!("{} {}", symbol, self.label),
            Style::default().fg(color),
        )
    }
}
