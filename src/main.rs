use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::*;
use compile::Compile;
use project::Project;

mod compile;
mod config;
mod project;
mod shell;
mod ui;
mod os;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    workdir: Option<String>,
    #[arg(short, long)]
    config: Option<String>,
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Build,
    Qemu(QemuArgs),
    Uboot,
}

#[derive(Args, Debug, Default)]
struct QemuArgs {
    #[arg(short, long)]
    debug: bool,
    #[arg(long)]
    dtb: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut workdir = PathBuf::from(cli.workdir.unwrap_or("./".to_string()));
    workdir = fs::canonicalize(&workdir)
        .unwrap_or_else(|_| panic!("Workdir not found: {}", workdir.display()));

    println!("Workdir: {}", workdir.display());

    let mut project = Project::new(workdir, cli.config)?;

    match cli.command {
        SubCommands::Build => {
            Compile::run(&mut project, false);
        }
        SubCommands::Qemu(args) => {
            Compile::run(&mut project, args.debug);
            
        }
        SubCommands::Uboot => {}
    }

    Ok(())
}