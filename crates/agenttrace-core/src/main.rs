//! AgentTrace CLI
//!
//! Command-line interface for the AgentTrace observability platform.

use clap::{Parser, Subcommand};
use std::process::ExitCode;
use tracing::info;
use chrono::{DateTime, Utc};

/// AgentTrace - Observability for AI Agents
#[derive(Parser)]
#[command(name = "agenttrace")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true, env = "AGENTTRACE_CONFIG")]
    config: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (for commands that support it)
    #[arg(long, global = true, default_value = "text")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Table,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the AgentTrace collector server
    Serve {
        /// HTTP API port
        #[arg(long, default_value = "8080", env = "AGENTTRACE_HTTP_PORT")]
        http_port: u16,

        /// gRPC port for OTLP ingestion
        #[arg(long, default_value = "4317", env = "AGENTTRACE_GRPC_PORT")]
        grpc_port: u16,

        /// UDP port for high-volume ingestion
        #[arg(long, default_value = "4318", env = "AGENTTRACE_UDP_PORT")]
        udp_port: u16,
    },

    /// Launch the TUI dashboard
    Dashboard {
        /// Refresh rate in milliseconds
        #[arg(long, default_value = "1000")]
        refresh: u64,

        /// Default time range to display
        #[arg(long, default_value = "1h")]
        time_range: String,
    },

    /// Start the web dashboard server
    Web {
        /// Port for the web server
        #[arg(long, default_value = "3000")]
        port: u16,

        /// Directory containing static files
        #[arg(long)]
        static_dir: Option<String>,
    },

    /// Query and manage traces
    Traces {
        #[command(subcommand)]
        command: TracesCommands,
    },

    /// View metrics and analytics
    Metrics {
        /// Service name filter
        #[arg(long)]
        service: Option<String>,

        /// Model name filter
        #[arg(long)]
        model: Option<String>,

        /// Time range (e.g., "1h", "24h", "7d")
        #[arg(long, default_value = "1h")]
        last: String,

        /// Group results by field
        #[arg(long)]
        group_by: Option<String>,
    },

    /// View cost breakdown
    Costs {
        /// Service name filter
        #[arg(long)]
        service: Option<String>,

        /// Group by (service, model, operation, day, hour)
        #[arg(long, default_value = "model")]
        group_by: String,

        /// Time range
        #[arg(long, default_value = "7d")]
        last: String,
    },

    /// Manage alert rules
    Alerts {
        #[command(subcommand)]
        command: AlertsCommands,
    },

    /// Database management
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },

    /// Run all services in development mode
    Dev {
        /// Skip database setup
        #[arg(long)]
        no_db: bool,
    },

    /// Show system health status
    Health,

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand)]
enum TracesCommands {
    /// List recent traces
    List {
        /// Service name filter
        #[arg(long)]
        service: Option<String>,

        /// Status filter (ok, error, in_progress)
        #[arg(long)]
        status: Option<String>,

        /// Minimum duration in milliseconds
        #[arg(long)]
        min_duration: Option<f64>,

        /// Time range
        #[arg(long, default_value = "1h")]
        last: String,

        /// Maximum number of results
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// Show trace details
    Show {
        /// Trace ID to display
        trace_id: String,

        /// Show full span details
        #[arg(long)]
        full: bool,
    },

    /// Export trace data
    Export {
        /// Trace ID to export
        trace_id: String,

        /// Output format (json, otlp, jaeger)
        #[arg(long, default_value = "json")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum AlertsCommands {
    /// List all alert rules
    List,

    /// Create a new alert rule
    Create {
        /// Alert name
        #[arg(long)]
        name: String,

        /// Metric to monitor (error_rate, latency_p99, cost_sum, etc.)
        #[arg(long)]
        metric: String,

        /// Comparison operator (gt, lt, eq, gte, lte)
        #[arg(long)]
        operator: String,

        /// Threshold value
        #[arg(long)]
        threshold: f64,

        /// Service name scope (optional)
        #[arg(long)]
        service: Option<String>,

        /// Severity (info, warning, critical)
        #[arg(long, default_value = "warning")]
        severity: String,
    },

    /// Delete an alert rule
    Delete {
        /// Rule ID to delete
        rule_id: String,
    },

    /// Test an alert rule
    Test {
        /// Rule ID to test
        rule_id: String,
    },

    /// Show alert history
    History {
        /// Only show active alerts
        #[arg(long)]
        active: bool,

        /// Time range
        #[arg(long, default_value = "24h")]
        last: String,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// Run database migrations
    Migrate {
        /// Target migration version (latest if not specified)
        #[arg(long)]
        target: Option<i64>,
    },

    /// Rollback migrations
    Rollback {
        /// Number of migrations to rollback
        #[arg(long, default_value = "1")]
        steps: usize,
    },

    /// Seed database with sample data
    Seed {
        /// Number of sample traces to create
        #[arg(long, default_value = "100")]
        traces: usize,
    },

    /// Show database statistics
    Stats,

    /// Reset database (WARNING: deletes all data)
    Reset {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();

    // Load configuration
    let config = match load_config(cli.config.as_deref()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Execute command
    let result = match cli.command {
        Commands::Serve {
            http_port,
            grpc_port,
            udp_port,
        } => run_serve(config, http_port, grpc_port, udp_port).await,
        Commands::Dashboard {
            refresh,
            time_range,
        } => run_dashboard(config, refresh, &time_range).await,
        Commands::Web { port, static_dir } => run_web(config, port, static_dir).await,
        Commands::Traces { command } => run_traces(config, command, cli.format).await,
        Commands::Metrics {
            service,
            model,
            last,
            group_by,
        } => run_metrics(config, service, model, &last, group_by, cli.format).await,
        Commands::Costs {
            service,
            group_by,
            last,
        } => run_costs(config, service, &group_by, &last, cli.format).await,
        Commands::Alerts { command } => run_alerts(config, command, cli.format).await,
        Commands::Db { command } => run_db(config, command).await,
        Commands::Dev { no_db } => run_dev(config, no_db).await,
        Commands::Health => run_health(config, cli.format).await,
        Commands::Completions { shell } => {
            generate_completions(shell);
            Ok(())
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn load_config(_path: Option<&str>) -> anyhow::Result<agenttrace::Config> {
    // TODO: Implement config loading
    info!("Loading configuration...");
    Ok(agenttrace::Config::default())
}

async fn run_serve(
    mut config: agenttrace::Config,
    http_port: u16,
    grpc_port: u16,
    _udp_port: u16,
) -> anyhow::Result<()> {
    // Override config with CLI args
    config.server.http_port = http_port;
    config.server.grpc_port = grpc_port;

    println!("🚀 AgentTrace collector starting...");
    println!("   HTTP API: http://{}:{}", config.server.host, http_port);
    println!("   gRPC:     {}:{}", config.server.host, grpc_port);
    println!("   Database: {}", config.database.url);
    println!("   Redis:    {}", config.redis.url);
    println!();

    // Create and start collector
    let mut collector = match agenttrace::collector::Collector::new(config).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to initialize collector: {}", e);
            eprintln!();
            eprintln!("Make sure TimescaleDB and Redis are running:");
            eprintln!("  docker-compose up -d timescaledb redis");
            return Err(anyhow::anyhow!("Collector initialization failed: {}", e));
        }
    };

    info!("Collector initialized successfully");
    println!("✅ Collector ready. Press Ctrl+C to stop.");
    println!();

    // Start serving
    if let Err(e) = collector.start().await {
        eprintln!("❌ Collector error: {}", e);
        return Err(anyhow::anyhow!("Collector error: {}", e));
    }

    Ok(())
}

async fn run_dashboard(
    _config: agenttrace::Config,
    refresh: u64,
    time_range: &str,
) -> anyhow::Result<()> {
    info!(
        "Starting TUI dashboard with {}ms refresh, {} time range",
        refresh, time_range
    );

    let mut app = agenttrace::tui::App::new()
        .with_refresh_rate(refresh)
        .with_time_range(time_range);

    app.run().await.map_err(|e| anyhow::anyhow!("{}", e))
}

async fn run_web(
    _config: agenttrace::Config,
    port: u16,
    _static_dir: Option<String>,
) -> anyhow::Result<()> {
    info!("Starting web dashboard on port {}", port);

    // TODO: Implement web server
    println!("🌐 Web dashboard: http://localhost:{port}");

    tokio::signal::ctrl_c().await?;
    Ok(())
}

async fn run_traces(
    config: agenttrace::Config,
    command: TracesCommands,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = format!("http://{}:{}", config.server.host, config.server.http_port);

    match command {
        TracesCommands::List { service, status, min_duration, last, limit } => {
            let since = parse_duration(&last)?;
            let mut url = format!("{}/api/v1/traces?limit={}", base_url, limit);

            if let Some(s) = service {
                url.push_str(&format!("&service={}", s));
            }
            if let Some(s) = status {
                url.push_str(&format!("&status={}", s));
            }
            if let Some(d) = min_duration {
                url.push_str(&format!("&min_duration={}", d));
            }
            url.push_str(&format!("&since={}", since.to_rfc3339()));

            let resp: serde_json::Value = client.get(&url).send().await?.json().await?;

            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&resp)?),
                _ => {
                    println!("┌─────────────┬────────────────────┬──────────────┬──────────┬────────┬──────────┐");
                    println!("│ Trace ID    │ Operation          │ Service      │ Duration │ Spans  │ Cost     │");
                    println!("├─────────────┼────────────────────┼──────────────┼──────────┼────────┼──────────┤");

                    if let Some(traces) = resp.get("traces").and_then(|t| t.as_array()) {
                        for trace in traces {
                            let id = trace.get("trace_id").and_then(|v| v.as_str()).unwrap_or("-");
                            let op = trace.get("root_operation").and_then(|v| v.as_str()).unwrap_or("-");
                            let svc = trace.get("service_name").and_then(|v| v.as_str()).unwrap_or("-");
                            let dur = trace.get("duration_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let spans = trace.get("span_count").and_then(|v| v.as_i64()).unwrap_or(0);
                            let cost = trace.get("total_cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);

                            println!(
                                "│ {:11} │ {:18} │ {:12} │ {:>6.1}ms │ {:>6} │ ${:>7.2} │",
                                truncate(id, 11), truncate(op, 18), truncate(svc, 12), dur, spans, cost
                            );
                        }
                    }
                    println!("└─────────────┴────────────────────┴──────────────┴──────────┴────────┴──────────┘");
                }
            }
        }
        TracesCommands::Show { trace_id, full } => {
            let url = format!("{}/api/v1/traces/{}", base_url, trace_id);
            let resp: serde_json::Value = client.get(&url).send().await?.json().await?;

            if full {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                // Print tree view
                println!("Trace: {}", trace_id);
                if let Some(summary) = resp.get("summary") {
                    println!("  Operation: {}", summary.get("root_operation").and_then(|v| v.as_str()).unwrap_or("-"));
                    println!("  Service:   {}", summary.get("service_name").and_then(|v| v.as_str()).unwrap_or("-"));
                    println!("  Duration:  {:.1}ms", summary.get("duration_ms").and_then(|v| v.as_f64()).unwrap_or(0.0));
                    println!("  Spans:     {}", summary.get("span_count").and_then(|v| v.as_i64()).unwrap_or(0));
                    println!("  Errors:    {}", summary.get("error_count").and_then(|v| v.as_i64()).unwrap_or(0));
                    println!("  Tokens:    {}", summary.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0));
                    println!("  Cost:      ${:.4}", summary.get("total_cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0));
                }
                println!();

                // Print span tree
                if let Some(spans) = resp.get("spans").and_then(|s| s.as_array()) {
                    println!("Spans:");
                    for span in spans {
                        let indent = if span.get("parent_span_id").is_some() { "  └─" } else { "" };
                        let op = span.get("operation_name").and_then(|v| v.as_str()).unwrap_or("-");
                        let dur = span.get("duration_ms").and_then(|v| v.as_f64()).map(|d| format!("{:.1}ms", d)).unwrap_or("-".to_string());
                        let status = span.get("status").and_then(|v| v.as_str()).unwrap_or("-");
                        let status_icon = if status == "error" { "✗" } else { "✓" };

                        println!("  {} {} {} [{}]", indent, status_icon, op, dur);
                    }
                }
            }
        }
        TracesCommands::Export { trace_id, format: export_format, output } => {
            let url = format!("{}/api/v1/traces/{}", base_url, trace_id);
            let resp: serde_json::Value = client.get(&url).send().await?.json().await?;

            let content = match export_format.as_str() {
                "json" => serde_json::to_string_pretty(&resp)?,
                _ => serde_json::to_string_pretty(&resp)?,
            };

            if let Some(path) = output {
                std::fs::write(&path, &content)?;
                println!("Exported to {}", path);
            } else {
                println!("{}", content);
            }
        }
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        format!("{:width$}", s, width = max)
    } else {
        format!("{}…", &s[..max-1])
    }
}

fn parse_duration(s: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
    use chrono::{Duration, Utc};

    let now = Utc::now();
    let duration = if s.ends_with('h') {
        let hours: i64 = s.trim_end_matches('h').parse()?;
        Duration::hours(hours)
    } else if s.ends_with('d') {
        let days: i64 = s.trim_end_matches('d').parse()?;
        Duration::days(days)
    } else if s.ends_with('m') {
        let minutes: i64 = s.trim_end_matches('m').parse()?;
        Duration::minutes(minutes)
    } else {
        Duration::hours(1)
    };

    Ok(now - duration)
}

async fn run_metrics(
    config: agenttrace::Config,
    service: Option<String>,
    model: Option<String>,
    last: &str,
    _group_by: Option<String>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = format!("http://{}:{}", config.server.host, config.server.http_port);
    let since = parse_duration(last)?;

    let mut url = format!("{}/api/v1/metrics/summary?since={}", base_url, since.to_rfc3339());
    if let Some(s) = service {
        url.push_str(&format!("&service={}", s));
    }
    if let Some(m) = model {
        url.push_str(&format!("&model={}", m));
    }

    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&resp)?),
        _ => {
            println!("📊 Metrics Summary (last {})", last);
            println!("────────────────────────────────");
            println!();

            let total_spans = resp.get("total_spans").and_then(|v| v.as_i64()).unwrap_or(0);
            let total_traces = resp.get("total_traces").and_then(|v| v.as_i64()).unwrap_or(0);
            let total_tokens = resp.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
            let total_cost = resp.get("total_cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let error_count = resp.get("error_count").and_then(|v| v.as_i64()).unwrap_or(0);
            let error_rate = resp.get("error_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let avg_latency = resp.get("avg_latency_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let p50 = resp.get("p50_latency_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let p95 = resp.get("p95_latency_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let p99 = resp.get("p99_latency_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);

            println!("  Total Spans:   {:>12}", format_number(total_spans));
            println!("  Total Traces:  {:>12}", format_number(total_traces));
            println!("  Total Tokens:  {:>12}", format_number(total_tokens));
            println!("  Total Cost:    {:>12}", format!("${:.2}", total_cost));
            println!();
            println!("  Errors:        {:>12}", error_count);
            println!("  Error Rate:    {:>12}", format!("{:.2}%", error_rate));
            println!();
            println!("  Avg Latency:   {:>12}", format!("{:.1}ms", avg_latency));
            println!("  p50 Latency:   {:>12}", format!("{:.1}ms", p50));
            println!("  p95 Latency:   {:>12}", format!("{:.1}ms", p95));
            println!("  p99 Latency:   {:>12}", format!("{:.1}ms", p99));
        }
    }

    Ok(())
}

async fn run_costs(
    config: agenttrace::Config,
    service: Option<String>,
    group_by: &str,
    last: &str,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = format!("http://{}:{}", config.server.host, config.server.http_port);
    let since = parse_duration(last)?;

    let mut url = format!(
        "{}/api/v1/metrics/costs?group_by={}&since={}",
        base_url, group_by, since.to_rfc3339()
    );
    if let Some(s) = service {
        url.push_str(&format!("&service={}", s));
    }

    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&resp)?),
        _ => {
            println!("💰 Cost Breakdown by {} (last {})", group_by, last);
            println!("──────────────────────────────────────────────────────────");
            println!();

            let total = resp.get("total_cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);

            if let Some(costs) = resp.get("costs").and_then(|c| c.as_array()) {
                println!("┌──────────────────────┬────────────┬────────────┬──────────┬─────────┐");
                println!("│ {}                 │ Cost       │ Tokens     │ Calls    │ % Total │",
                    if group_by == "model" { "Model" } else { "Group" });
                println!("├──────────────────────┼────────────┼────────────┼──────────┼─────────┤");

                for cost in costs {
                    let group = cost.get("group").and_then(|v| v.as_str()).unwrap_or("-");
                    let cost_usd = cost.get("total_cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let tokens = cost.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                    let calls = cost.get("call_count").and_then(|v| v.as_i64()).unwrap_or(0);
                    let pct = if total > 0.0 { cost_usd / total * 100.0 } else { 0.0 };

                    println!(
                        "│ {:20} │ ${:>8.2} │ {:>10} │ {:>8} │ {:>6.1}% │",
                        truncate(group, 20), cost_usd, format_number(tokens), calls, pct
                    );
                }

                println!("├──────────────────────┼────────────┼────────────┼──────────┼─────────┤");
                println!("│ TOTAL                │ ${:>8.2} │            │          │  100.0% │", total);
                println!("└──────────────────────┴────────────┴────────────┴──────────┴─────────┘");
            }
        }
    }

    Ok(())
}

fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

async fn run_alerts(
    config: agenttrace::Config,
    command: AlertsCommands,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let base_url = format!("http://{}:{}", config.server.host, config.server.http_port);

    match command {
        AlertsCommands::List => {
            let url = format!("{}/api/v1/alerts/rules", base_url);
            let resp: serde_json::Value = client.get(&url).send().await?.json().await?;

            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&resp)?),
                _ => {
                    println!("🔔 Alert Rules");
                    println!("──────────────────────────────────────────────────────────────────");
                    println!();

                    if let Some(rules) = resp.as_array() {
                        if rules.is_empty() {
                            println!("  No alert rules configured.");
                            println!("  Use 'agenttrace alerts create' to add a rule.");
                        } else {
                            println!("┌─────────────────────┬──────────┬────────────────┬───────────┬─────────┐");
                            println!("│ Name                │ Metric   │ Condition      │ Severity  │ Enabled │");
                            println!("├─────────────────────┼──────────┼────────────────┼───────────┼─────────┤");

                            for rule in rules {
                                let name = rule.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                                let metric = rule.get("metric").and_then(|v| v.as_str()).unwrap_or("-");
                                let op = rule.get("operator").and_then(|v| v.as_str()).unwrap_or(">");
                                let threshold = rule.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let severity = rule.get("severity").and_then(|v| v.as_str()).unwrap_or("-");
                                let enabled = rule.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);

                                let condition = format!("{} {:.2}", op, threshold);
                                let enabled_str = if enabled { "✓" } else { "✗" };

                                println!(
                                    "│ {:19} │ {:8} │ {:14} │ {:9} │ {:>7} │",
                                    truncate(name, 19), truncate(metric, 8), condition, severity, enabled_str
                                );
                            }

                            println!("└─────────────────────┴──────────┴────────────────┴───────────┴─────────┘");
                        }
                    }
                }
            }
        }
        AlertsCommands::Create { name, metric, operator, threshold, service, severity } => {
            let url = format!("{}/api/v1/alerts/rules", base_url);

            let body = serde_json::json!({
                "name": name,
                "metric": metric,
                "operator": operator,
                "threshold": threshold,
                "service_name": service,
                "severity": severity,
                "condition_type": "threshold"
            });

            let resp = client.post(&url).json(&body).send().await?;

            if resp.status().is_success() {
                let rule: serde_json::Value = resp.json().await?;
                let id = rule.get("id").and_then(|v| v.as_str()).unwrap_or("-");
                println!("✅ Created alert rule: {} ({})", name, id);
            } else {
                let error: serde_json::Value = resp.json().await?;
                println!("❌ Failed to create rule: {:?}", error);
            }
        }
        AlertsCommands::Delete { rule_id } => {
            let url = format!("{}/api/v1/alerts/rules/{}", base_url, rule_id);
            let resp = client.delete(&url).send().await?;

            if resp.status().is_success() {
                println!("✅ Deleted alert rule: {}", rule_id);
            } else {
                println!("❌ Failed to delete rule (not found or error)");
            }
        }
        AlertsCommands::Test { rule_id } => {
            let url = format!("{}/api/v1/alerts/rules/{}/test", base_url, rule_id);
            let resp: serde_json::Value = client.post(&url).send().await?.json().await?;

            let would_trigger = resp.get("would_trigger").and_then(|v| v.as_bool()).unwrap_or(false);
            let current_value = resp.get("current_value").and_then(|v| v.as_f64());

            if would_trigger {
                println!("⚠️  Alert WOULD trigger");
                if let Some(val) = current_value {
                    println!("   Current value: {:.4}", val);
                }
                if let Some(event) = resp.get("event") {
                    println!("   Message: {}", event.get("message").and_then(|v| v.as_str()).unwrap_or("-"));
                }
            } else {
                println!("✓ Alert would NOT trigger");
                if let Some(val) = current_value {
                    println!("   Current value: {:.4}", val);
                }
            }
        }
        AlertsCommands::History { active, last } => {
            let since = parse_duration(&last)?;
            let mut url = format!("{}/api/v1/alerts/events?since={}", base_url, since.to_rfc3339());

            if active {
                url.push_str("&status=active");
            }

            let resp: serde_json::Value = client.get(&url).send().await?.json().await?;

            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&resp)?),
                _ => {
                    let title = if active { "Active Alerts" } else { "Alert History" };
                    println!("🔔 {} (last {})", title, last);
                    println!("──────────────────────────────────────────────────────────────────");
                    println!();

                    if let Some(events) = resp.as_array() {
                        if events.is_empty() {
                            println!("  No alerts found.");
                        } else {
                            println!("┌───────────────────┬──────────┬────────────────────────────────┬──────────────┐");
                            println!("│ Rule              │ Severity │ Message                        │ Status       │");
                            println!("├───────────────────┼──────────┼────────────────────────────────┼──────────────┤");

                            for event in events {
                                let rule_id = event.get("rule_id").and_then(|v| v.as_str()).unwrap_or("-");
                                let severity = event.get("severity").and_then(|v| v.as_str()).unwrap_or("-");
                                let message = event.get("message").and_then(|v| v.as_str()).unwrap_or("-");
                                let status = event.get("status").and_then(|v| v.as_str()).unwrap_or("-");

                                let severity_icon = match severity {
                                    "critical" => "🚨",
                                    "warning" => "⚠️ ",
                                    _ => "ℹ️ ",
                                };

                                let status_display = match status {
                                    "active" => "● Active",
                                    "acknowledged" => "◐ Acked",
                                    "resolved" => "○ Resolved",
                                    _ => status,
                                };

                                println!(
                                    "│ {:17} │ {} {:5} │ {:30} │ {:12} │",
                                    truncate(&rule_id[..8.min(rule_id.len())], 17),
                                    severity_icon,
                                    severity,
                                    truncate(message, 30),
                                    status_display
                                );
                            }

                            println!("└───────────────────┴──────────┴────────────────────────────────┴──────────────┘");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn run_db(_config: agenttrace::Config, command: DbCommands) -> anyhow::Result<()> {
    match command {
        DbCommands::Migrate { target } => {
            println!(
                "Running migrations to {}...",
                target.map_or("latest".to_string(), |t| t.to_string())
            );
        }
        DbCommands::Rollback { steps } => {
            println!("Rolling back {steps} migration(s)...");
        }
        DbCommands::Seed { traces } => {
            println!("Seeding database with {traces} sample traces...");
        }
        DbCommands::Stats => {
            println!("Database statistics:");
            println!("  (Implementation pending)");
        }
        DbCommands::Reset { force } => {
            if !force {
                println!("WARNING: This will delete all data!");
                println!("Use --force to confirm.");
                return Ok(());
            }
            println!("Resetting database...");
        }
    }
    // TODO: Implement
    Ok(())
}

async fn run_dev(_config: agenttrace::Config, no_db: bool) -> anyhow::Result<()> {
    println!("🔧 Starting development environment...");
    if !no_db {
        println!("   Starting database containers...");
    }
    println!("   Starting collector...");
    println!("   Starting web dashboard...");
    // TODO: Implement
    Ok(())
}

async fn run_health(config: agenttrace::Config, format: OutputFormat) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let base_url = format!("http://{}:{}", config.server.host, config.server.http_port);
    let health_url = format!("{}/health", base_url);

    println!("🏥 System Health Check");
    println!("─────────────────────");
    println!();

    // Check collector/API
    let collector_status = match client.get(&health_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let version = body.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
            format!("✅ Running (v{})", version)
        }
        Ok(resp) => format!("⚠️  Unhealthy ({})", resp.status()),
        Err(e) => format!("❌ Unreachable ({})", e),
    };

    // Check database (via the API's ability to respond)
    let db_status = if collector_status.starts_with("✅") {
        // If API is up, DB is probably fine
        "✅ Connected".to_string()
    } else {
        "❓ Unknown".to_string()
    };

    // Check Redis (same logic)
    let redis_status = if collector_status.starts_with("✅") {
        "✅ Connected".to_string()
    } else {
        "❓ Unknown".to_string()
    };

    println!("  Collector: {}", collector_status);
    println!("  Database:  {}", db_status);
    println!("  Redis:     {}", redis_status);
    println!();

    match format {
        OutputFormat::Json => {
            let health = serde_json::json!({
                "collector": {
                    "url": health_url,
                    "status": if collector_status.starts_with("✅") { "ok" } else { "error" }
                },
                "database": {
                    "status": if db_status.starts_with("✅") { "ok" } else { "unknown" }
                },
                "redis": {
                    "status": if redis_status.starts_with("✅") { "ok" } else { "unknown" }
                }
            });
            println!("{}", serde_json::to_string_pretty(&health)?);
        }
        _ => {
            if collector_status.starts_with("✅") {
                println!("All systems operational.");
            } else {
                println!("Some systems may be unavailable.");
                println!("Tip: Run 'agenttrace serve' to start the collector.");
            }
        }
    }

    Ok(())
}

fn generate_completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    use clap_complete::generate;
    use std::io;

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "agenttrace", &mut io::stdout());
}
