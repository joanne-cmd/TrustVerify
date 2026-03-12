//! TrustVerify - PPID Verification Dashboard
//! Backend API and CLI.

mod api;

use clap::Parser;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use trustverify::verifier::verify;
use trustverify::Registry;

#[derive(clap::Parser)]
#[command(name = "trustverify")]
#[command(about = "Verify attestation quotes against provider registry")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Start the API server
    Serve {
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long, default_value = "8080")]
        port: u16,
        #[arg(long, default_value = "../registry.json")]
        registry: PathBuf,
    },
    /// Verify a quote from file or stdin
    Verify {
        /// Path to quote file (hex) or "-" for stdin
        #[arg(long, short)]
        quote: String,
        /// Path to registry JSON
        #[arg(long, default_value = "../registry.json")]
        registry: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Serve { host, port, registry }) => {
            let registry = Arc::new(load_registry(&registry)?);
            api::serve(host, port, registry).await?;
        }
        Some(Command::Verify { quote, registry }) => {
            let quote_content = if quote == "-" {
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf
            } else {
                std::fs::read_to_string(&quote).map_err(|e| format!("Failed to read {}: {}", quote, e))?
            };
            let result = verify(quote_content.trim(), &registry);
            println!("{}", serde_json::to_string_pretty(&result)?);
            match result.status.as_str() {
                "Trusted" => std::process::exit(0),
                "Unknown" => std::process::exit(1),
                _ => std::process::exit(2),
            }
        }
        None => {
            // Default: serve (run from backend/ with cd backend && cargo run)
            let registry_path = PathBuf::from("../registry.json");
            let registry_path = if registry_path.exists() {
                registry_path
            } else {
                PathBuf::from("registry.json")
            };
            let registry = Arc::new(load_registry(&registry_path)?);
            api::serve("0.0.0.0".to_string(), 8080, registry).await?;
        }
    }

    Ok(())
}

fn load_registry(path: &PathBuf) -> Result<Registry, Box<dyn std::error::Error + Send + Sync>> {
    trustverify::registry::load_registry(path)
}
