use clap::{Parser, Subcommand};
use hotwired_mcp::{ipc, server};
use rmcp::{transport::stdio, ServiceExt};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::Mutex;

/// Hotwired MCP server for Claude Code integration
#[derive(Parser, Debug)]
#[command(name = "hotwired-mcp")]
#[command(about = "MCP server for Hotwired multi-agent workflows")]
struct Args {
    /// Path to the Unix socket for communicating with the Hotwired backend.
    /// Defaults to ~/.hotwired/hotwired.sock. Only use this for worktree development.
    #[arg(long, short = 's', global = true)]
    socket_path: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Register a Claude session with the Hotwired backend (called by SessionStart hook)
    Register {
        /// Zellij session name
        #[arg(long)]
        session: String,
        /// Project directory path
        #[arg(long)]
        project: String,
    },
    /// Deregister a Claude session (called by SessionEnd hook)
    Deregister {
        /// Zellij session name
        #[arg(long)]
        session: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Socket path: CLI arg only, otherwise use default (~/.hotwired/hotwired.sock)
    // NOTE: We intentionally do NOT read HOTWIRED_SOCKET_PATH env var here.
    // This prevents worktree environments from accidentally overriding the socket path
    // when Claude sessions should connect to the main Hotwired backend.
    let socket_path = args.socket_path.clone();

    // Handle subcommands (register/deregister are quick CLI operations, not MCP servers)
    if let Some(cmd) = args.command {
        return handle_command(cmd, socket_path).await;
    }

    // No subcommand: Start the MCP server
    // Initialize logging - MUST NOT write to stdout
    // stdout is used for JSON-RPC communication with the MCP client
    // Write logs to a file in ~/.hotwired/logs/ directory

    // Determine log directory from socket path or use default
    let log_dir = if let Some(ref path) = socket_path {
        // Use parent directory of socket path (e.g., ~/.hotwired/)
        PathBuf::from(path)
            .parent()
            .map(|p| p.join("logs"))
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join(".hotwired/logs")
            })
    } else {
        // Default to ~/.hotwired/logs/
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".hotwired/logs")
    };

    // Create logs directory if it doesn't exist
    std::fs::create_dir_all(&log_dir).ok();

    // Open log file with line buffering for immediate writes
    let log_path = log_dir.join("mcp-rs.log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    if let Some(file) = log_file {
        tracing_subscriber::fmt()
            .with_writer(Mutex::new(file))
            .with_ansi(false)
            .init();
    } else {
        // Fallback to stderr (never stdout)
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .init();
    }

    tracing::info!("Hotwired MCP server starting");
    if let Some(ref path) = socket_path {
        tracing::info!("Using socket path: {}", path);
    }

    // Create IPC client to communicate with hotwired-core via Unix socket
    // Reads HOTWIRED_SOCKET_PATH env var for worktree support (default: ~/.hotwired/hotwired.sock)
    let client = ipc::UnixSocketClient::new(socket_path);

    // Create and run the server with STDIO transport
    let service = server::HotwiredMcp::new(client)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            eprintln!("Error starting server: {}", e);
        })?;

    service.waiting().await?;

    Ok(())
}

/// Handle register/deregister subcommands
async fn handle_command(
    cmd: Command,
    socket_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create IPC client
    let client = ipc::UnixSocketClient::new(socket_path);

    match cmd {
        Command::Register { session, project } => {
            // Send register request to backend
            match client.register_session(&session, &project).await {
                Ok(_) => {
                    // Success - exit silently (this runs as a hook)
                    Ok(())
                }
                Err(e) => {
                    // Log error to stderr (hooks should fail silently)
                    eprintln!("Failed to register session: {}", e);
                    Ok(())
                }
            }
        }
        Command::Deregister { session } => {
            // Send deregister request to backend
            match client.deregister_session(&session).await {
                Ok(_) => {
                    // Success - exit silently
                    Ok(())
                }
                Err(e) => {
                    // Log error to stderr (hooks should fail silently)
                    eprintln!("Failed to deregister session: {}", e);
                    Ok(())
                }
            }
        }
    }
}
