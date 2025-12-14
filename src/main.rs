use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ai;
mod cli;
mod config;
mod doc;
mod error;
mod git;
mod mcp;
mod storage;

use error::Result;

#[derive(Parser)]
#[command(name = "ktme")]
#[command(author, version, about = "Knowledge Transfer Me - Automated documentation generation", long_about = None)]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    quiet: bool,

    #[arg(long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract code changes from various sources
    Extract {
        #[arg(long, group = "source")]
        commit: Option<String>,

        #[arg(long, group = "source")]
        staged: bool,

        #[arg(long, group = "source")]
        pr: Option<u32>,

        #[arg(long, requires = "pr")]
        provider: Option<String>,

        #[arg(short, long)]
        output: Option<String>,
    },

    /// Generate documentation from code changes
    Generate {
        #[arg(long, group = "source")]
        commit: Option<String>,

        #[arg(long, group = "source")]
        input: Option<String>,

        #[arg(long, group = "source")]
        pr: Option<u32>,

        #[arg(long, group = "source")]
        staged: bool,

        #[arg(long, required = true)]
        service: String,

        #[arg(long)]
        r#type: Option<String>,

        #[arg(long)]
        format: Option<String>,

        #[arg(long)]
        output: Option<String>,

        #[arg(long)]
        template: Option<String>,
    },

    /// Update existing documentation
    Update {
        #[arg(long, group = "source")]
        commit: Option<String>,

        #[arg(long, group = "source")]
        pr: Option<u32>,

        #[arg(long, group = "source")]
        staged: bool,

        #[arg(long, required = true)]
        service: String,

        #[arg(long)]
        section: Option<String>,

        #[arg(long)]
        dry_run: bool,
    },

    /// Manage service-to-document mappings
    Mapping {
        #[command(subcommand)]
        command: MappingCommands,
    },

    /// MCP server management
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum MappingCommands {
    /// Add a service mapping
    Add {
        service: String,
        #[arg(long, group = "location")]
        url: Option<String>,
        #[arg(long, group = "location")]
        file: Option<String>,
    },

    /// List all service mappings
    List {
        #[arg(long)]
        service: Option<String>,
    },

    /// Get mapping for a specific service
    Get { service: String },

    /// Remove a service mapping
    Remove { service: String },

    /// Discover services automatically
    Discover {
        #[arg(long)]
        directory: String,
    },

    /// Edit mappings file
    Edit,
}

#[derive(Subcommand)]
enum McpCommands {
    /// Start MCP server
    Start {
        #[arg(long)]
        config: Option<String>,

        #[arg(long)]
        daemon: bool,

        #[arg(long)]
        stdio: bool,
    },

    /// Check MCP server status
    Status,

    /// Stop MCP server
    Stop,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Initialize configuration
    Init,

    /// Show current configuration
    Show,

    /// Set configuration value
    Set { key: String, value: String },

    /// Validate configuration
    Validate,
}

fn setup_logging(verbose: bool, quiet: bool, is_stdio: bool) {
    // Skip logging entirely in STDIO mode to avoid JSON parsing issues
    if is_stdio {
        return;
    }

    let log_level = if quiet {
        tracing::Level::ERROR
    } else if verbose {
        tracing::Level::DEBUG
    } else {
        std::env::var("KTME_LOG_LEVEL")
            .ok()
            .and_then(|l| l.parse().ok())
            .unwrap_or(tracing::Level::INFO)
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("ktme={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check if we're in stdio mode for MCP
    let is_stdio = if let Commands::Mcp { command: McpCommands::Start { stdio, .. } } = &cli.command {
        *stdio
    } else {
        false
    };

    setup_logging(cli.verbose, cli.quiet, is_stdio);

    // Only log if not in stdio mode
    if !is_stdio {
        tracing::info!("Starting ktme v{}", env!("CARGO_PKG_VERSION"));
    }

    match cli.command {
        Commands::Extract {
            commit,
            staged,
            pr,
            provider,
            output,
        } => {
            cli::commands::extract::execute(commit, staged, pr, provider, output).await?;
        }
        Commands::Generate {
            commit,
            input,
            pr,
            staged,
            service,
            r#type,
            format,
            output,
            template,
        } => {
            cli::commands::generate::execute(
                commit, input, pr, staged, service, r#type, format, output, template,
            )
            .await?;
        }
        Commands::Update {
            commit,
            pr,
            staged,
            service,
            section,
            dry_run,
        } => {
            cli::commands::update::execute(commit, pr, staged, service, section, dry_run).await?;
        }
        Commands::Mapping { command } => match command {
            MappingCommands::Add { service, url, file } => {
                cli::commands::mapping::add(service, url, file).await?;
            }
            MappingCommands::List { service } => {
                cli::commands::mapping::list(service).await?;
            }
            MappingCommands::Get { service } => {
                cli::commands::mapping::get(service).await?;
            }
            MappingCommands::Remove { service } => {
                cli::commands::mapping::remove(service).await?;
            }
            MappingCommands::Discover { directory } => {
                cli::commands::mapping::discover(directory).await?;
            }
            MappingCommands::Edit => {
                cli::commands::mapping::edit().await?;
            }
        },
        Commands::Mcp { command } => match command {
            McpCommands::Start { config, daemon, stdio } => {
                cli::commands::mcp::start(config, daemon, stdio).await?;
            }
            McpCommands::Status => {
                cli::commands::mcp::status().await?;
            }
            McpCommands::Stop => {
                cli::commands::mcp::stop().await?;
            }
        },
        Commands::Config { command } => match command {
            ConfigCommands::Init => {
                cli::commands::config::init().await?;
            }
            ConfigCommands::Show => {
                cli::commands::config::show().await?;
            }
            ConfigCommands::Set { key, value } => {
                cli::commands::config::set(key, value).await?;
            }
            ConfigCommands::Validate => {
                cli::commands::config::validate().await?;
            }
        },
    }

    Ok(())
}
