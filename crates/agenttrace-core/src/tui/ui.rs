//! UI rendering for the TUI

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};

use super::app::{ActiveTab, App};
use crate::models::SpanStatus;

/// Main colors
const PRIMARY: Color = Color::Cyan;
const SECONDARY: Color = Color::Magenta;
const SUCCESS: Color = Color::Green;
const WARNING: Color = Color::Yellow;
const ERROR: Color = Color::Red;
const MUTED: Color = Color::DarkGray;

/// Draw the entire UI
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header + tabs
            Constraint::Min(10),    // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.size());

    draw_header(frame, app, chunks[0]);
    draw_content(frame, app, chunks[1]);
    draw_status_bar(frame, app, chunks[2]);

    // Draw help overlay if active
    if app.show_help {
        draw_help_overlay(frame);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Min(40),
            Constraint::Length(20),
        ])
        .split(area);

    // Logo
    let logo = Paragraph::new("🔭 AgentTrace")
        .style(Style::default().fg(PRIMARY).bold())
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(logo, chunks[0]);

    // Tabs
    let tabs = vec!["Overview", "Traces", "Costs", "Alerts", "Search"];
    let tab_titles: Vec<Line> = tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if i == app.active_tab.index() {
                Style::default().fg(PRIMARY).bold()
            } else {
                Style::default().fg(MUTED)
            };
            Line::from(format!(" {} {} ", i + 1, t)).style(style)
        })
        .collect();

    let tabs_widget = Tabs::new(tab_titles)
        .select(app.active_tab.index())
        .style(Style::default())
        .highlight_style(Style::default().fg(PRIMARY))
        .divider(symbols::line::VERTICAL);

    frame.render_widget(tabs_widget, chunks[1]);

    // Connection status
    let status = if app.connected {
        Span::styled("● Connected", Style::default().fg(SUCCESS))
    } else {
        Span::styled("○ Disconnected", Style::default().fg(ERROR))
    };
    let status_widget = Paragraph::new(status)
        .alignment(Alignment::Right)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(status_widget, chunks[2]);
}

fn draw_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        ActiveTab::Overview => draw_overview(frame, app, area),
        ActiveTab::Traces => draw_traces(frame, app, area),
        ActiveTab::Costs => draw_costs(frame, app, area),
        ActiveTab::Alerts => draw_alerts(frame, app, area),
        ActiveTab::Search => draw_search(frame, app, area),
    }
}

fn draw_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),   // Metrics cards
            Constraint::Length(10),  // Sparklines
            Constraint::Min(10),     // Recent activity
        ])
        .split(area);

    // Metric cards
    draw_metric_cards(frame, app, chunks[0]);

    // Sparklines
    draw_sparklines(frame, app, chunks[1]);

    // Recent activity split
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[2]);

    draw_recent_spans(frame, app, bottom_chunks[0]);
    draw_costs_summary(frame, app, bottom_chunks[1]);
}

fn draw_metric_cards(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(area);

    let cards = [
        ("Traces", format!("{}", app.metrics.total_traces), PRIMARY),
        ("Tokens", format_number(app.metrics.total_tokens), SECONDARY),
        ("Cost", format!("${:.2}", app.metrics.total_cost_usd), SUCCESS),
        ("Errors", format!("{}", app.metrics.error_count), if app.metrics.error_count > 0 { ERROR } else { MUTED }),
        ("Avg Latency", format!("{:.0}ms", app.metrics.avg_latency_ms), WARNING),
    ];

    for (i, (title, value, color)) in cards.iter().enumerate() {
        let block = Block::default()
            .title(*title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MUTED));

        let text = Paragraph::new(value.as_str())
            .style(Style::default().fg(*color).bold())
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(text, chunks[i]);
    }
}

fn draw_sparklines(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Tokens per minute sparkline
    let tokens_block = Block::default()
        .title("Tokens/min (last hour)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MUTED));

    let tokens_sparkline = Sparkline::default()
        .block(tokens_block)
        .data(&app.tokens_sparkline)
        .style(Style::default().fg(PRIMARY));

    frame.render_widget(tokens_sparkline, chunks[0]);

    // Cost sparkline
    let cost_data: Vec<u64> = app.cost_sparkline.iter().map(|x| (*x * 100.0) as u64).collect();
    let cost_block = Block::default()
        .title("Cost/hour (last 24h)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MUTED));

    let cost_sparkline = Sparkline::default()
        .block(cost_block)
        .data(&cost_data)
        .style(Style::default().fg(SUCCESS));

    frame.render_widget(cost_sparkline, chunks[1]);
}

fn draw_recent_spans(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Recent Activity")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MUTED));

    let header = Row::new(vec!["Operation", "Type", "Duration", "Tokens", "Status"])
        .style(Style::default().fg(PRIMARY).bold())
        .height(1);

    let rows: Vec<Row> = app
        .recent_spans
        .iter()
        .take(10)
        .map(|span| {
            let status_style = match span.status {
                SpanStatus::Ok => Style::default().fg(SUCCESS),
                SpanStatus::Error => Style::default().fg(ERROR),
                _ => Style::default().fg(MUTED),
            };

            Row::new(vec![
                Cell::from(truncate(&span.operation, 20)),
                Cell::from(span.span_type.clone()),
                Cell::from(span.duration_ms.map_or("-".to_string(), |d| format!("{:.0}ms", d))),
                Cell::from(span.tokens.map_or("-".to_string(), |t| t.to_string())),
                Cell::from(format!("{:?}", span.status)).style(status_style),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

fn draw_costs_summary(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Cost by Model")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MUTED));

    let total_cost: f64 = app.costs_by_model.iter().map(|c| c.cost_usd).sum();

    let rows: Vec<Row> = app
        .costs_by_model
        .iter()
        .map(|cost| {
            let percentage = if total_cost > 0.0 {
                (cost.cost_usd / total_cost * 100.0) as u16
            } else {
                0
            };

            let bar = "█".repeat((percentage / 5) as usize);

            Row::new(vec![
                Cell::from(truncate(&cost.model, 15)),
                Cell::from(format!("${:.2}", cost.cost_usd)),
                Cell::from(bar).style(Style::default().fg(SECONDARY)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(25),
            Constraint::Percentage(35),
        ],
    )
    .block(block);

    frame.render_widget(table, area);
}

fn draw_traces(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!("Traces (last {})", app.time_range))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MUTED));

    let header = Row::new(vec!["Trace ID", "Operation", "Service", "Duration", "Spans", "Tokens", "Cost", "Status"])
        .style(Style::default().fg(PRIMARY).bold())
        .height(1);

    let rows: Vec<Row> = app
        .traces
        .iter()
        .map(|trace| {
            let status_style = match trace.status {
                SpanStatus::Ok => Style::default().fg(SUCCESS),
                SpanStatus::Error => Style::default().fg(ERROR),
                _ => Style::default().fg(MUTED),
            };

            Row::new(vec![
                Cell::from(truncate(&trace.trace_id, 10)),
                Cell::from(truncate(&trace.operation, 15)),
                Cell::from(truncate(&trace.service, 12)),
                Cell::from(format_duration(trace.duration_ms)),
                Cell::from(trace.span_count.to_string()),
                Cell::from(format_number(trace.tokens as u64)),
                Cell::from(format!("${:.2}", trace.cost_usd)),
                Cell::from(format!("{:?}", trace.status)).style(status_style),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Percentage(14),
            Constraint::Percentage(12),
            Constraint::Percentage(8),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(16),
        ],
    )
    .header(header)
    .block(block)
    .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(table, area, &mut app.traces_state.clone());
}

fn draw_costs(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(10)])
        .split(area);

    // Summary
    let summary_block = Block::default()
        .title("Cost Summary")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MUTED));

    let total_cost: f64 = app.costs_by_model.iter().map(|c| c.cost_usd).sum();
    let total_tokens: u64 = app.costs_by_model.iter().map(|c| c.tokens).sum();
    let total_calls: u64 = app.costs_by_model.iter().map(|c| c.call_count).sum();

    let summary_text = vec![
        Line::from(vec![
            Span::raw("Total Cost: "),
            Span::styled(format!("${:.2}", total_cost), Style::default().fg(SUCCESS).bold()),
        ]),
        Line::from(vec![
            Span::raw("Total Tokens: "),
            Span::styled(format_number(total_tokens), Style::default().fg(PRIMARY)),
        ]),
        Line::from(vec![
            Span::raw("Total Calls: "),
            Span::styled(format_number(total_calls), Style::default().fg(SECONDARY)),
        ]),
        Line::from(vec![
            Span::raw("Avg Cost/Call: "),
            Span::styled(
                format!("${:.4}", if total_calls > 0 { total_cost / total_calls as f64 } else { 0.0 }),
                Style::default().fg(WARNING),
            ),
        ]),
    ];

    let summary = Paragraph::new(summary_text)
        .block(summary_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(summary, chunks[0]);

    // Detail table
    let detail_block = Block::default()
        .title("Cost by Model")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MUTED));

    let header = Row::new(vec!["Model", "Provider", "Tokens", "Calls", "Cost", "% of Total"])
        .style(Style::default().fg(PRIMARY).bold())
        .height(1);

    let rows: Vec<Row> = app
        .costs_by_model
        .iter()
        .map(|cost| {
            let percentage = if total_cost > 0.0 {
                cost.cost_usd / total_cost * 100.0
            } else {
                0.0
            };

            Row::new(vec![
                Cell::from(cost.model.clone()),
                Cell::from(cost.provider.clone()),
                Cell::from(format_number(cost.tokens)),
                Cell::from(cost.call_count.to_string()),
                Cell::from(format!("${:.2}", cost.cost_usd)),
                Cell::from(format!("{:.1}%", percentage)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(12),
            Constraint::Percentage(15),
            Constraint::Percentage(18),
        ],
    )
    .header(header)
    .block(detail_block);

    frame.render_widget(table, chunks[1]);
}

fn draw_alerts(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)])
        .split(area);

    // Alert summary
    let active_count = app.alerts.iter().filter(|a| a.status == "active").count();
    let summary_style = if active_count > 0 {
        Style::default().fg(WARNING)
    } else {
        Style::default().fg(SUCCESS)
    };

    let summary_text = if active_count > 0 {
        format!("⚠ {} active alert(s)", active_count)
    } else {
        "✓ No active alerts".to_string()
    };

    let summary = Paragraph::new(summary_text)
        .style(summary_style.bold())
        .block(
            Block::default()
                .title("Alert Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(MUTED)),
        );

    frame.render_widget(summary, chunks[0]);

    // Alerts table
    let block = Block::default()
        .title("Alert History")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MUTED));

    let header = Row::new(vec!["Rule", "Severity", "Message", "Triggered", "Status"])
        .style(Style::default().fg(PRIMARY).bold())
        .height(1);

    let rows: Vec<Row> = app
        .alerts
        .iter()
        .map(|alert| {
            let severity_style = match alert.severity.as_str() {
                "critical" => Style::default().fg(ERROR).bold(),
                "warning" => Style::default().fg(WARNING),
                _ => Style::default().fg(MUTED),
            };

            let status_style = match alert.status.as_str() {
                "active" => Style::default().fg(ERROR),
                "acknowledged" => Style::default().fg(WARNING),
                "resolved" => Style::default().fg(SUCCESS),
                _ => Style::default().fg(MUTED),
            };

            Row::new(vec![
                Cell::from(alert.rule_name.clone()),
                Cell::from(alert.severity.clone()).style(severity_style),
                Cell::from(truncate(&alert.message, 40)),
                Cell::from(alert.triggered_at.clone()),
                Cell::from(alert.status.clone()).style(status_style),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(18),
            Constraint::Percentage(12),
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(block)
    .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(table, area, &mut app.alerts_state.clone());
}

fn draw_search(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)])
        .split(area);

    // Search input
    let search_style = if app.search_focused {
        Style::default().fg(PRIMARY)
    } else {
        Style::default().fg(MUTED)
    };

    let cursor = if app.search_focused { "▌" } else { "" };
    let search_text = format!("🔍 {}{}", app.search_query, cursor);

    let search_input = Paragraph::new(search_text)
        .style(search_style)
        .block(
            Block::default()
                .title(if app.search_focused { "Search (Enter to search, Esc to cancel)" } else { "Search (/ to focus)" })
                .borders(Borders::ALL)
                .border_style(search_style),
        );

    frame.render_widget(search_input, chunks[0]);

    // Search results or help
    if app.search_results.is_empty() {
        let help_text = vec![
            Line::from("Search Syntax:").style(Style::default().fg(PRIMARY).bold()),
            Line::from(""),
            Line::from("  service:my-agent     Filter by service name"),
            Line::from("  model:claude-opus    Filter by model"),
            Line::from("  status:error         Filter by status (ok, error)"),
            Line::from("  duration:>1000       Filter by duration (ms)"),
            Line::from("  cost:>0.1            Filter by cost (USD)"),
            Line::from("  operation:llm_call   Filter by operation name"),
            Line::from(""),
            Line::from("  Combine with spaces: service:my-agent status:error"),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("Search Help")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(MUTED)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(help, chunks[1]);
    } else {
        // Show search results as a traces table
        let block = Block::default()
            .title(format!("Results ({})", app.search_results.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(MUTED));

        let header = Row::new(vec!["Trace ID", "Operation", "Service", "Duration", "Cost", "Status"])
            .style(Style::default().fg(PRIMARY).bold())
            .height(1);

        let rows: Vec<Row> = app
            .search_results
            .iter()
            .map(|trace| {
                let status_style = match trace.status {
                    SpanStatus::Ok => Style::default().fg(SUCCESS),
                    SpanStatus::Error => Style::default().fg(ERROR),
                    _ => Style::default().fg(MUTED),
                };

                Row::new(vec![
                    Cell::from(truncate(&trace.trace_id, 12)),
                    Cell::from(truncate(&trace.operation, 20)),
                    Cell::from(truncate(&trace.service, 15)),
                    Cell::from(format_duration(trace.duration_ms)),
                    Cell::from(format!("${:.2}", trace.cost_usd)),
                    Cell::from(format!("{:?}", trace.status)).style(status_style),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(15),
                Constraint::Percentage(25),
                Constraint::Percentage(18),
                Constraint::Percentage(15),
                Constraint::Percentage(12),
                Constraint::Percentage(15),
            ],
        )
        .header(header)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_stateful_widget(table, chunks[1], &mut app.search_state.clone());
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Status message or default help
    let left_text = app.get_status().unwrap_or("? Help | Tab Switch | q Quit");
    let left = Paragraph::new(left_text)
        .style(Style::default().fg(MUTED));
    frame.render_widget(left, chunks[0]);

    // Time range and refresh info
    let right_text = format!(
        "Range: {} | Refresh: {:?} | Last: {}",
        app.time_range,
        app.refresh_rate,
        format_elapsed(app.last_update.elapsed())
    );
    let right = Paragraph::new(right_text)
        .style(Style::default().fg(MUTED))
        .alignment(Alignment::Right);
    frame.render_widget(right, chunks[1]);
}

fn draw_help_overlay(frame: &mut Frame) {
    let area = centered_rect(60, 70, frame.size());

    // Clear the background
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from("Keyboard Shortcuts").style(Style::default().fg(PRIMARY).bold()),
        Line::from(""),
        Line::from("Navigation:").style(Style::default().fg(SECONDARY)),
        Line::from("  Tab / Shift+Tab    Switch between tabs"),
        Line::from("  1-5                Jump to specific tab"),
        Line::from("  j/k or ↑/↓         Navigate lists"),
        Line::from("  Enter              Select item"),
        Line::from(""),
        Line::from("Search:").style(Style::default().fg(SECONDARY)),
        Line::from("  /                  Focus search"),
        Line::from("  Esc                Cancel search"),
        Line::from(""),
        Line::from("Alerts:").style(Style::default().fg(SECONDARY)),
        Line::from("  a                  Acknowledge selected alert"),
        Line::from(""),
        Line::from("General:").style(Style::default().fg(SECONDARY)),
        Line::from("  ?                  Toggle this help"),
        Line::from("  q / Ctrl+C         Quit"),
        Line::from(""),
        Line::from("Press any key to close").style(Style::default().fg(MUTED).italic()),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PRIMARY)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(help, area);
}

// Helper functions

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_duration(ms: f64) -> String {
    if ms >= 60_000.0 {
        format!("{:.1}m", ms / 60_000.0)
    } else if ms >= 1_000.0 {
        format!("{:.1}s", ms / 1_000.0)
    } else {
        format!("{:.0}ms", ms)
    }
}

fn format_elapsed(elapsed: std::time::Duration) -> String {
    let secs = elapsed.as_secs();
    if secs < 60 {
        format!("{}s ago", secs)
    } else {
        format!("{}m ago", secs / 60)
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
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
