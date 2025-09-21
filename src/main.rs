use clap::{Parser, Subcommand};
use std::process::{Command, exit};
use tokio::process::Command as AsyncCommand;

#[derive(Parser)]
#[command(name = "taskline")]
#[command(about = "Ultra-fast scripting framework for high-performance task automation")]
#[command(version = "0.1.0")]
#[command(author = "d33p0st")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new taskline script
    #[command(name = "init")]
    Init {
        /// Name of the script file to create
        filename: String,
        /// Optional version (e.g., v1.0.0)
        version: Option<String>,
    },
    /// Bump version of an existing taskline script
    #[command(name = "bump")]
    Bump {
        /// Script file to bump version for
        filename: String,
        /// Version bump type
        #[arg(value_enum)]
        bump_type: Option<BumpType>,
    },
    /// Install all Taskline components (taskline-init, taskline-bump)
    #[command(name = "install")]
    Install {
        /// Force reinstall even if already installed
        #[arg(long)]
        force: bool,
    },
    /// Check status of Taskline components
    #[command(name = "doctor")]
    Doctor,
}

#[derive(clap::ValueEnum, Clone)]
enum BumpType {
    Major,
    Minor,
    Patch,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { filename, version } => {
            // Route to taskline-init binary
            let mut cmd = Command::new("taskline-init");
            cmd.arg(&filename);
            
            if let Some(ver) = version {
                cmd.arg(&ver);
            }
            
            execute_command(cmd, "taskline-init").await;
        }
        Commands::Bump { filename, bump_type } => {
            // Route to taskline-bump binary
            let mut cmd = Command::new("taskline-bump");
            cmd.arg(&filename);
            
            if let Some(bt) = bump_type {
                match bt {
                    BumpType::Major => cmd.arg("major"),
                    BumpType::Minor => cmd.arg("minor"),
                    BumpType::Patch => cmd.arg("patch"),
                };
            }
            
            execute_command(cmd, "taskline-bump").await;
        }
        Commands::Install { force } => {
            install_components(force).await;
        }
        Commands::Doctor => {
            check_installation().await;
        }
    }
}

async fn execute_command(mut cmd: Command, binary_name: &str) {
    match cmd.status() {
        Ok(status) => {
            if !status.success() {
                if let Some(code) = status.code() {
                    exit(code);
                } else {
                    eprintln!("❌ {} was terminated by signal", binary_name);
                    exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to execute {}: {}", binary_name, e);
            eprintln!("💡 Component not found. Run 'taskline install' to install all components");
            exit(1);
        }
    }
}

async fn install_components(force: bool) {
    println!("🚀 Installing Taskline components...");
    
    let components = vec![
        ("taskline-init", "Script initialization tool"),
        ("taskline-bump", "Version bumping tool"),
    ];
    
    for (component, description) in components {
        println!("📦 Installing {} ({})...", component, description);
        
        let mut cmd = AsyncCommand::new("cargo");
        cmd.args(&["install", component]);
        
        if force {
            cmd.arg("--force");
        }
        
        match cmd.status().await {
            Ok(status) => {
                if status.success() {
                    println!("✅ {} installed successfully", component);
                } else {
                    eprintln!("❌ Failed to install {}", component);
                    exit(1);
                }
            }
            Err(e) => {
                eprintln!("❌ Error installing {}: {}", component, e);
                exit(1);
            }
        }
    }
    
    println!("🎉 All Taskline components installed successfully!");
    println!("");
    println!("Available commands:");
    println!("  taskline init <filename> [version]  - Initialize a new script");
    println!("  taskline bump <filename> [type]     - Bump script version");
    println!("  taskline doctor                     - Check installation status");
}

async fn check_installation() {
    println!("🔍 Checking Taskline installation...");
    println!("");
    
    let components = vec![
        ("taskline", "Main CLI dispatcher"),
        ("taskline-init", "Script initialization tool"),
        ("taskline-bump", "Version bumping tool"),
    ];
    
    let mut all_installed = true;
    
    for (component, description) in components {
        print!("  {} ({})... ", component, description);
        
        let mut cmd = AsyncCommand::new("which");
        cmd.arg(component);
        
        match cmd.status().await {
            Ok(status) if status.success() => {
                println!("✅ Installed");
            }
            _ => {
                println!("❌ Not found");
                all_installed = false;
            }
        }
    }
    
    println!("");
    
    if all_installed {
        println!("🎉 All Taskline components are properly installed!");
    } else {
        println!("⚠️  Some components are missing. Run 'taskline install' to install them.");
    }
}