// SPDX-FileCopyrightText: 2025 Madeline Baggins <declanbaggins@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-only

#[derive(clap::Parser)]
pub struct Args {
    #[arg(long, default_value = "./config.xml")]
    pub config: std::path::PathBuf,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    Init {
        #[command(subcommand)]
        command: InitCommands,
    },
    Switch,
    Search {
        #[arg(default_value = "")]
        search: String,
    },
}

#[derive(clap::Subcommand)]
pub enum InitCommands {
    Server(InitServerArgs),
}

#[derive(clap::Args)]
pub struct InitServerArgs {
    #[arg(long)]
    pub symlinks: std::path::PathBuf,
    #[arg(long)]
    pub store: std::path::PathBuf,
    #[arg(long)]
    pub branch: String,
}
