#[derive(clap::Parser)]
pub struct Args {
    #[arg(short, default_value = "./config.xml")]
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
}
