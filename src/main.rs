mod cli {
    #[derive(clap::Parser)]
    pub struct Args {
        #[arg(long, default_value = "smartget.toml")]
        pub config: std::path::PathBuf,
        #[command(subcommand)]
        pub command: Subcommand,
    }

    #[derive(clap::Subcommand)]
    pub enum Subcommand {
        Check,
        Example,
    }
}

mod config {
    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct Config {
        pub server: Vec<Server>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                server: vec![Server::default()],
            }
        }
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct Server {
        pub root: url::Url,
        #[serde(alias = "repository")]
        pub repositories: Vec<Repository>,
    }

    impl Default for Server {
        fn default() -> Self {
            Server {
                root: "git+ssh:://user@example.com"
                    .try_into()
                    .unwrap(),
                repositories: vec![Repository::default()],
            }
        }
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct Repository {
        pub path: String,
    }

    impl Default for Repository {
        fn default() -> Self {
            Repository {
                path: "example".to_string(),
            }
        }
    }
}

pub mod esc {
    pub const RED: &str = "\x1b[1;31m";
    pub const DEFAULT: &str = "\x1b[1;39m";
}

enum Error {
    CouldNotOpenFile(std::path::PathBuf),
    InvalidToml(std::path::PathBuf, toml::de::Error),
}

impl std::fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        use esc::*;
        match self {
            Error::CouldNotOpenFile(file) => write!(
                f,
                "Could not open file \"{}\"",
                file.display()
            ),
            Error::InvalidToml(file, error) => write!(
                f,
                "Invalid toml in file \"{}\"\n{RED}Internal Error{DEFAULT}:\n{:?}",
                file.display(),
                error,
            ),
        }
    }
}

fn handle_command(
    command: cli::Subcommand,
    config: std::path::PathBuf,
) -> Result<(), Error> {
    match command {
        cli::Subcommand::Check => handle_check(config),
        cli::Subcommand::Example => handle_example(),
    }
}

fn handle_check(
    config: std::path::PathBuf,
) -> Result<(), Error> {
    let toml_txt = std::fs::read_to_string(&config)
        .map_err(|_| {
            Error::CouldNotOpenFile(config.clone())
        })?;
    let _: config::Config = toml::from_str(&toml_txt)
        .map_err(|e| Error::InvalidToml(config, e))?;
    Ok(())
}

fn handle_example() -> Result<(), Error> {
    println!(
        "{}",
        &toml::to_string_pretty(&config::Config::default())
            .unwrap()
    );
    Ok(())
}

fn main() {
    use clap::Parser;
    let args = cli::Args::parse();
    if let Err(err) =
        handle_command(args.command, args.config)
    {
        use esc::*;
        println!("{RED}Error{DEFAULT}: {err}");
    }
}
