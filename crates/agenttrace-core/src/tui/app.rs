//! Main TUI application state and logic

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::widgets::TableState;

use crate::models::{Span, SpanStatus};

/// Active view/tab in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTab {
    #[default]
    Overview,
    Traces,
    Costs,
    Alerts,
    Search,
}

impl ActiveTab {
    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::Traces,
            Self::Traces => Self::Costs,
            Self::Costs => Self::Alerts,
            Self::Alerts => Self::Search,
            Self::Search => Self::Overview,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Overview => Self::Search,
            Self::Traces => Self::Overview,
            Self::Costs => Self::Traces,
            Self::Alerts => Self::Costs,
            Self::Search => Self::Alerts,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Overview => 0,
            Self::Traces => 1,
            Self::Costs => 2,
            Self::Alerts => 3,
            Self::Search => 4,
        }
    }
}

/// Summary metrics for display
#[derive(Debug, Clone, Default)]
pub struct MetricsSummary {
    pub total_traces: u64,
    pub total_spans: u64,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub error_count: u64,
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub spans_per_minute: f64,
}

/// Cost breakdown by model
#[derive(Debug, Clone)]
pub struct CostByModel {
    pub model: String,
    pub provider: String,
    pub tokens: u64,
    pub cost_usd: f64,
    pub call_count: u64,
}

/// Alert event for display
#[derive(Debug, Clone)]
pub struct AlertDisplay {
    pub id: String,
    pub rule_name: String,
    pub severity: String,
    pub message: String,
    pub triggered_at: String,
    pub status: String,
}

/// Trace summary for list display
#[derive(Debug, Clone)]
pub struct TraceSummary {
    pub trace_id: String,
    pub operation: String,
    pub service: String,
    pub duration_ms: f64,
    pub span_count: u32,
    pub tokens: u32,
    pub cost_usd: f64,
    pub status: SpanStatus,
    pub started_at: String,
}

/// Recent span for real-time display
#[derive(Debug, Clone)]
pub struct RecentSpan {
    pub span_id: String,
    pub trace_id: String,
    pub operation: String,
    pub span_type: String,
    pub duration_ms: Option<f64>,
    pub tokens: Option<u32>,
    pub status: SpanStatus,
    pub timestamp: String,
}

/// Main TUI application state
pub struct App {
    /// Whether the app should quit
    pub should_quit: bool,
    /// Active tab
    pub active_tab: ActiveTab,
    /// Summary metrics
    pub metrics: MetricsSummary,
    /// Cost breakdown by model
    pub costs_by_model: Vec<CostByModel>,
    /// Recent traces
    pub traces: Vec<TraceSummary>,
    /// Recent spans (live feed)
    pub recent_spans: Vec<RecentSpan>,
    /// Active alerts
    pub alerts: Vec<AlertDisplay>,
    /// Search query
    pub search_query: String,
    /// Search results
    pub search_results: Vec<TraceSummary>,
    /// Is search input focused
    pub search_focused: bool,
    /// Traces table state
    pub traces_state: TableState,
    /// Spans table state
    pub spans_state: TableState,
    /// Alerts table state
    pub alerts_state: TableState,
    /// Search results table state
    pub search_state: TableState,
    /// Last update time
    pub last_update: Instant,
    /// Refresh rate
    pub refresh_rate: Duration,
    /// Selected time range (e.g., "1h", "24h", "7d")
    pub time_range: String,
    /// Show help overlay
    pub show_help: bool,
    /// Status message
    pub status_message: Option<(String, Instant)>,
    /// Connection status
    pub connected: bool,
    /// Sparkline data for tokens/minute
    pub tokens_sparkline: Vec<u64>,
    /// Sparkline data for cost/hour
    pub cost_sparkline: Vec<f64>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new TUI app
    pub fn new() -> Self {
        Self {
            should_quit: false,
            active_tab: ActiveTab::default(),
            metrics: MetricsSummary::default(),
            costs_by_model: Vec::new(),
            traces: Vec::new(),
            recent_spans: Vec::new(),
            alerts: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_focused: false,
            traces_state: TableState::default(),
            spans_state: TableState::default(),
            alerts_state: TableState::default(),
            search_state: TableState::default(),
            last_update: Instant::now(),
            refresh_rate: Duration::from_secs(1),
            time_range: "1h".to_string(),
            show_help: false,
            status_message: None,
            connected: false,
            tokens_sparkline: vec![0; 60],
            cost_sparkline: vec![0.0; 24],
        }
    }

    /// Set refresh rate
    pub fn with_refresh_rate(mut self, ms: u64) -> Self {
        self.refresh_rate = Duration::from_millis(ms);
        self
    }

    /// Set time range
    pub fn with_time_range(mut self, range: &str) -> Self {
        self.time_range = range.to_string();
        self
    }

    /// Handle key events
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        // Global shortcuts
        match (code, modifiers) {
            (KeyCode::Char('q'), KeyModifiers::NONE) if !self.search_focused => {
                self.should_quit = true;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (KeyCode::Char('?'), KeyModifiers::NONE) if !self.search_focused => {
                self.show_help = !self.show_help;
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                if self.show_help {
                    self.show_help = false;
                } else if self.search_focused {
                    self.search_focused = false;
                }
            }
            (KeyCode::Tab, KeyModifiers::NONE) if !self.search_focused => {
                self.active_tab = self.active_tab.next();
            }
            (KeyCode::BackTab, KeyModifiers::SHIFT) if !self.search_focused => {
                self.active_tab = self.active_tab.prev();
            }
            (KeyCode::Char('1'), KeyModifiers::NONE) if !self.search_focused => {
                self.active_tab = ActiveTab::Overview;
            }
            (KeyCode::Char('2'), KeyModifiers::NONE) if !self.search_focused => {
                self.active_tab = ActiveTab::Traces;
            }
            (KeyCode::Char('3'), KeyModifiers::NONE) if !self.search_focused => {
                self.active_tab = ActiveTab::Costs;
            }
            (KeyCode::Char('4'), KeyModifiers::NONE) if !self.search_focused => {
                self.active_tab = ActiveTab::Alerts;
            }
            (KeyCode::Char('5'), KeyModifiers::NONE) if !self.search_focused => {
                self.active_tab = ActiveTab::Search;
            }
            (KeyCode::Char('/'), KeyModifiers::NONE) if !self.search_focused => {
                self.active_tab = ActiveTab::Search;
                self.search_focused = true;
            }
            _ => {
                // Tab-specific handling
                self.handle_tab_key(code, modifiers);
            }
        }
    }

    fn handle_tab_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) {
        match self.active_tab {
            ActiveTab::Traces => self.handle_traces_key(code),
            ActiveTab::Alerts => self.handle_alerts_key(code),
            ActiveTab::Search => self.handle_search_key(code),
            _ => {}
        }
    }

    fn handle_traces_key(&mut self, code: KeyCode) {
        let len = self.traces.len();
        if len == 0 {
            return;
        }

        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.traces_state.selected().unwrap_or(0);
                self.traces_state.select(Some(i.saturating_sub(1)));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.traces_state.selected().unwrap_or(0);
                self.traces_state.select(Some((i + 1).min(len - 1)));
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.traces_state.select(Some(0));
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.traces_state.select(Some(len - 1));
            }
            KeyCode::Enter => {
                if let Some(idx) = self.traces_state.selected() {
                    if let Some(trace) = self.traces.get(idx) {
                        self.set_status(format!("Selected trace: {}", trace.trace_id));
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_alerts_key(&mut self, code: KeyCode) {
        let len = self.alerts.len();
        if len == 0 {
            return;
        }

        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.alerts_state.selected().unwrap_or(0);
                self.alerts_state.select(Some(i.saturating_sub(1)));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.alerts_state.selected().unwrap_or(0);
                self.alerts_state.select(Some((i + 1).min(len - 1)));
            }
            KeyCode::Char('a') => {
                self.set_status("Acknowledged alert".to_string());
            }
            _ => {}
        }
    }

    fn handle_search_key(&mut self, code: KeyCode) {
        if self.search_focused {
            match code {
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                KeyCode::Enter => {
                    self.search_focused = false;
                    self.set_status(format!("Searching for: {}", self.search_query));
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                _ => {}
            }
        } else {
            match code {
                KeyCode::Char('i') | KeyCode::Char('/') => {
                    self.search_focused = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let len = self.search_results.len();
                    if len > 0 {
                        let i = self.search_state.selected().unwrap_or(0);
                        self.search_state.select(Some(i.saturating_sub(1)));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let len = self.search_results.len();
                    if len > 0 {
                        let i = self.search_state.selected().unwrap_or(0);
                        self.search_state.select(Some((i + 1).min(len - 1)));
                    }
                }
                _ => {}
            }
        }
    }

    /// Set a status message that expires after 3 seconds
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some((message, Instant::now()));
    }

    /// Get current status message if not expired
    pub fn get_status(&self) -> Option<&str> {
        self.status_message.as_ref().and_then(|(msg, time)| {
            if time.elapsed() < Duration::from_secs(3) {
                Some(msg.as_str())
            } else {
                None
            }
        })
    }

    /// Update with new span data
    pub fn add_span(&mut self, span: RecentSpan) {
        self.recent_spans.insert(0, span);
        if self.recent_spans.len() > 100 {
            self.recent_spans.pop();
        }
        self.metrics.total_spans += 1;
    }

    /// Update metrics
    pub fn update_metrics(&mut self, metrics: MetricsSummary) {
        self.metrics = metrics;
        self.last_update = Instant::now();
    }

    /// Check if data needs refresh
    pub fn needs_refresh(&self) -> bool {
        self.last_update.elapsed() >= self.refresh_rate
    }

    /// Load sample data for demo
    pub fn load_demo_data(&mut self) {
        self.connected = true;

        // Sample metrics
        self.metrics = MetricsSummary {
            total_traces: 1_234,
            total_spans: 45_678,
            total_tokens: 2_345_678,
            total_cost_usd: 127.45,
            error_count: 23,
            avg_latency_ms: 234.5,
            p99_latency_ms: 1_250.0,
            spans_per_minute: 156.7,
        };

        // Sample costs by model
        self.costs_by_model = vec![
            CostByModel {
                model: "claude-opus-4".to_string(),
                provider: "anthropic".to_string(),
                tokens: 1_200_000,
                cost_usd: 89.50,
                call_count: 234,
            },
            CostByModel {
                model: "claude-sonnet-4".to_string(),
                provider: "anthropic".to_string(),
                tokens: 800_000,
                cost_usd: 28.40,
                call_count: 567,
            },
            CostByModel {
                model: "gpt-4o".to_string(),
                provider: "openai".to_string(),
                tokens: 345_678,
                cost_usd: 9.55,
                call_count: 123,
            },
        ];

        // Sample traces
        self.traces = vec![
            TraceSummary {
                trace_id: "abc123".to_string(),
                operation: "code_review".to_string(),
                service: "review-agent".to_string(),
                duration_ms: 45_230.0,
                span_count: 23,
                tokens: 12_456,
                cost_usd: 0.89,
                status: SpanStatus::Ok,
                started_at: "2 min ago".to_string(),
            },
            TraceSummary {
                trace_id: "def456".to_string(),
                operation: "bug_fix".to_string(),
                service: "coding-agent".to_string(),
                duration_ms: 123_450.0,
                span_count: 45,
                tokens: 34_567,
                cost_usd: 2.34,
                status: SpanStatus::Ok,
                started_at: "5 min ago".to_string(),
            },
            TraceSummary {
                trace_id: "ghi789".to_string(),
                operation: "test_generation".to_string(),
                service: "test-agent".to_string(),
                duration_ms: 67_890.0,
                span_count: 12,
                tokens: 8_901,
                cost_usd: 0.45,
                status: SpanStatus::Error,
                started_at: "8 min ago".to_string(),
            },
        ];

        // Sample recent spans
        self.recent_spans = vec![
            RecentSpan {
                span_id: "span1".to_string(),
                trace_id: "abc123".to_string(),
                operation: "llm_call".to_string(),
                span_type: "llm".to_string(),
                duration_ms: Some(1_234.0),
                tokens: Some(456),
                status: SpanStatus::Ok,
                timestamp: "just now".to_string(),
            },
            RecentSpan {
                span_id: "span2".to_string(),
                trace_id: "abc123".to_string(),
                operation: "tool:read_file".to_string(),
                span_type: "tool".to_string(),
                duration_ms: Some(45.0),
                tokens: None,
                status: SpanStatus::Ok,
                timestamp: "1s ago".to_string(),
            },
        ];

        // Sample alerts
        self.alerts = vec![
            AlertDisplay {
                id: "alert1".to_string(),
                rule_name: "High Error Rate".to_string(),
                severity: "warning".to_string(),
                message: "Error rate above 5% for review-agent".to_string(),
                triggered_at: "10 min ago".to_string(),
                status: "active".to_string(),
            },
        ];

        // Sample sparkline data
        self.tokens_sparkline = vec![
            120, 145, 167, 189, 156, 178, 190, 210, 234, 256,
            245, 230, 210, 189, 167, 145, 156, 178, 190, 210,
            234, 256, 278, 290, 310, 289, 267, 245, 234, 212,
            190, 178, 167, 156, 145, 134, 123, 145, 167, 189,
            210, 234, 256, 278, 300, 289, 267, 245, 223, 201,
            189, 178, 167, 189, 210, 234, 256, 278, 290, 310,
        ];

        self.cost_sparkline = vec![
            2.3, 2.5, 2.8, 3.1, 2.9, 2.7, 2.5, 2.8, 3.2, 3.5,
            3.8, 4.1, 3.9, 3.6, 3.3, 3.0, 2.8, 2.6, 2.9, 3.2,
            3.5, 3.8, 4.0, 4.2,
        ];

        self.traces_state.select(Some(0));
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> crate::error::Result<()> {
        use crossterm::{
            execute,
            terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        };
        use ratatui::{backend::CrosstermBackend, Terminal};
        use std::io;

        // Setup terminal
        enable_raw_mode().map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;

        // Load demo data for now
        self.load_demo_data();

        // Create event handler
        let mut events = super::EventHandler::new(self.refresh_rate.as_millis() as u64);
        events.start();

        // Main loop
        while !self.should_quit {
            // Draw UI
            terminal
                .draw(|frame| super::ui::draw(frame, self))
                .map_err(|e| crate::error::Error::Tui(e.to_string()))?;

            // Handle events
            if let Some(event) = events.next().await {
                match event {
                    super::Event::Key(key) => {
                        self.handle_key(key.code, key.modifiers);
                    }
                    super::Event::Tick => {
                        // Periodic updates would go here
                    }
                    super::Event::Resize(_, _) => {
                        // Terminal handles resize automatically
                    }
                    _ => {}
                }
            }
        }

        // Restore terminal
        disable_raw_mode().map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;
        terminal.show_cursor()
            .map_err(|e| crate::error::Error::Tui(e.to_string()))?;

        Ok(())
    }
}
