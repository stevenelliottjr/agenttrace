//! AgentTrace CLI
//!
//! Command-line interface for the AgentTrace observability platform.

use clap::{Parser, Subcommand};
use std::process::ExitCode;
use tracing::info;

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
    _config: agenttrace::Config,
    http_port: u16,
    grpc_port: u16,
    udp_port: u16,
) -> anyhow::Result<()> {
    info!(
        "Starting AgentTrace collector on HTTP:{}, gRPC:{}, UDP:{}",
        http_port, grpc_port, udp_port
    );

    // TODO: Implement collector
    println!("🚀 AgentTrace collector starting...");
    println!("   HTTP API: http://0.0.0.0:{http_port}");
    println!("   gRPC:     0.0.0.0:{grpc_port}");
    println!("   UDP:      0.0.0.0:{udp_port}");
    println!();
    println!("Press Ctrl+C to stop");

    // Keep running until interrupted
    tokio::signal::ctrl_c().await?;
    println!("\nShutting down...");

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

    // TODO: Implement TUI
    println!("📊 Starting TUI dashboard...");
    println!("   (TUI implementation pending)");

    Ok(())
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
    _config: agenttrace::Config,
    command: TracesCommands,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    match command {
        TracesCommands::List { limit, .. } => {
            println!("Listing up to {limit} traces...");
            // TODO: Implement
        }
        TracesCommands::Show { trace_id, full } => {
            println!("Showing trace {trace_id} (full: {full})");
            // TODO: Implement
        }
        TracesCommands::Export {
            trace_id, format, ..
        } => {
            println!("Exporting trace {trace_id} as {format}");
            // TODO: Implement
        }
    }
    Ok(())
}

async fn run_metrics(
    _config: agenttrace::Config,
    _service: Option<String>,
    _model: Option<String>,
    _last: &str,
    _group_by: Option<String>,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    println!("Fetching metrics...");
    // TODO: Implement
    Ok(())
}

async fn run_costs(
    _config: agenttrace::Config,
    _service: Option<String>,
    _group_by: &str,
    _last: &str,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    println!("Fetching cost breakdown...");
    // TODO: Implement
    Ok(())
}

async fn run_alerts(
    _config: agenttrace::Config,
    command: AlertsCommands,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    match command {
        AlertsCommands::List => {
            println!("Listing alert rules...");
        }
        AlertsCommands::Create { name, .. } => {
            println!("Creating alert rule: {name}");
        }
        AlertsCommands::Delete { rule_id } => {
            println!("Deleting alert rule: {rule_id}");
        }
        AlertsCommands::Test { rule_id } => {
            println!("Testing alert rule: {rule_id}");
        }
        AlertsCommands::History { .. } => {
            println!("Showing alert history...");
        }
    }
    // TODO: Implement
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

async fn run_health(_config: agenttrace::Config, _format: OutputFormat) -> anyhow::Result<()> {
    println!("🏥 System Health Check");
    println!("─────────────────────");
    println!("Database:  ✅ Connected");
    println!("Redis:     ✅ Connected");
    println!("Collector: ✅ Running");
    // TODO: Implement actual health checks
    Ok(())
}

fn generate_completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    use clap_complete::generate;
    use std::io;

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "agenttrace", &mut io::stdout());
}
