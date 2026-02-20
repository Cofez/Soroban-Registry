mod commands;
mod config;
mod export;
mod import;
mod manifest;
mod patch; // Added to ensure patch module is found
mod wizard;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::patch::Severity;

#[derive(Parser)]
#[command(name = "soroban-registry")]
#[command(about = "A package manager for Soroban smart contracts", version)]
struct Cli {
    #[arg(long, default_value = "http://localhost:3001")]
    api_url: String,

    #[arg(long)]
    network: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for contracts
    Search { query: String, #[arg(long)] verified_only: bool },
    /// Get contract details
    Info { contract_id: String },
    /// Publish a contract
    Publish {
        contract_id: String,
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        tags: Option<String>,
        #[arg(long)]
        publisher: String,
    },
    /// List recent contracts
    List { #[arg(long, default_value_t = 10)] limit: usize },
    /// Migrate a contract
    Migrate {
        contract_id: String,
        wasm: String,
        #[arg(long)] simulate_fail: bool,
        #[arg(long)] dry_run: bool,
    },
    /// Export contract archive
    Export { id: String, output: String, contract_dir: String },
    /// Import contract archive
    Import { archive: String, output_dir: String },
    /// Generate documentation from contract
    Doc { contract_path: String, #[arg(long, default_value = "docs")] output: String },
    /// Run setup wizard
    Wizard {},
    /// View command history
    History { search: Option<String>, #[arg(long, default_value_t = 10)] limit: usize },
    /// Manage security patches
    Patch { #[command(subcommand)] action: PatchCommands },
}

#[derive(Subcommand)]
pub enum PatchCommands {
    Create { version: String, hash: String, severity: String, rollout: u8 },
    Notify { patch_id: String },
    Apply { contract_id: String, patch_id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Resolve network configuration (Handles "auto" routing logic from config.rs)
    let network = config::resolve_network(cli.network)?;

    match cli.command {
        Commands::Search { query, verified_only } => {
            commands::search(&cli.api_url, &query, network, verified_only).await?;
        }
        Commands::Info { contract_id } => {
            commands::info(&cli.api_url, &contract_id, network).await?;
        }
        Commands::Publish {
            contract_id,
            name,
            description,
            category,
            tags,
            publisher,
        } => {
            let tags_vec = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            commands::publish(
                &cli.api_url,
                &contract_id,
                &name,
                description.as_deref(),
                network,
                category.as_deref(),
                tags_vec,
                &publisher,
            )
            .await?;
        }
        Commands::List { limit } => {
            commands::list(&cli.api_url, limit, network).await?;
        }
        Commands::Migrate {
            contract_id,
            wasm,
            simulate_fail,
            dry_run,
        } => {
            commands::migrate(&cli.api_url, &contract_id, &wasm, simulate_fail, dry_run).await?;
        }
        Commands::Export { id, output, contract_dir } => {
            commands::export(&cli.api_url, &id, &output, &contract_dir).await?;
        }
        Commands::Import { archive, output_dir } => {
            commands::import(&cli.api_url, &archive, network, &output_dir).await?;
        }
        Commands::Doc { contract_path, output } => {
            commands::doc(&contract_path, &output)?;
        }
        Commands::Wizard {} => {
            wizard::run(&cli.api_url).await?;
        }
        Commands::History { search, limit } => {
            wizard::show_history(search.as_deref(), limit)?;
        }
        Commands::Patch { action } => match action {
            PatchCommands::Create { version, hash, severity, rollout } => {
                let sev = severity.parse::<Severity>()?;
                commands::patch_create(&cli.api_url, &version, &hash, sev, rollout).await?;
            }
            PatchCommands::Notify { patch_id } => {
                commands::patch_notify(&cli.api_url, &patch_id).await?;
            }
            PatchCommands::Apply { contract_id, patch_id } => {
                commands::patch_apply(&cli.api_url, &contract_id, &patch_id).await?;
            }
        },
    }
    Ok(())
}