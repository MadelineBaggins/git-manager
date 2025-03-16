use std::{fs::File, io::Read, path::PathBuf};

use cfg::Config;
use clap::Parser as _;
use maddi_xml::{Content, FromElement, Parser};

mod cfg;

mod cli {
    #[derive(clap::Parser)]
    pub struct Args {
        #[arg(short, default_value = "./config.xml")]
        pub config: std::path::PathBuf,
        #[command(subcommand)]
        pub command: Commands,
    }

    #[derive(clap::Subcommand)]
    pub enum Commands {
        Init,
    }
}

const RED: &str = "\x1b[1;31m";
const DEFAULT: &str = "\x1b[1;39m";

pub enum Error {
    FailedToOpenConfig(PathBuf),
    FailedToReadConfig(PathBuf),
    MaddiXml(String),
    FailedToInitRepository(PathBuf),
    FailedToConfigureRepository(PathBuf),
}

impl<'a> From<maddi_xml::Error<'a>> for Error {
    fn from(value: maddi_xml::Error<'a>) -> Self {
        Self::MaddiXml(format!("{value}"))
    }
}

impl std::fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Error::FailedToOpenConfig(path) => {
                writeln!(f, "{RED}Error:{DEFAULT}")?;
                write!(
                    f,
                    "failed to open config file '{}'",
                    path.display()
                )
            }
            Error::FailedToReadConfig(path) => {
                writeln!(f, "{RED}Error:{DEFAULT}")?;
                write!(
                    f,
                    "failed to read config file '{}'",
                    path.display()
                )
            }
            Error::MaddiXml(raw) => write!(f, "{raw}"),
            Error::FailedToInitRepository(path) => {
                writeln!(f, "{RED}Error:{DEFAULT}")?;
                write!(
                    f,
                    "failed to create repository '{}'",
                    path.display()
                )
            }
            Error::FailedToConfigureRepository(path) => {
                writeln!(f, "{RED}Error:{DEFAULT}")?;
                write!(
                    f,
                    "failed to configure repository '{}'",
                    path.display()
                )
            }
        }
    }
}

impl Config {
    fn load(path: PathBuf) -> Result<Self, Error> {
        // Open the configuration file
        let Ok(mut file) = File::open(&path) else {
            return Err(Error::FailedToOpenConfig(path));
        };
        // Read in the configuration file
        let mut source = String::new();
        if file.read_to_string(&mut source).is_err() {
            return Err(Error::FailedToReadConfig(path));
        };
        // Create the parser
        let mut parser = Parser::new(&path, &source);
        // Get the first piece of content in the file
        let content =
            parser.parse::<Option<Result<Content, maddi_xml::Error>>>().transpose()?;
        // Ensure the content was an element named 'config'
        let element = match content {
            Some(Content::Element(e)) => {
                if e.name == "config" {
                    e
                } else {
                    return Err(e
                        .position
                        .error(
                            "expected 'config' element"
                                .into(),
                        )
                        .into());
                }
            }
            _ => {
                return Err(parser
                    .position
                    .error(
                        "expected 'config' element".into(),
                    )
                    .into())
            }
        };
        // Get the config from the xml ast
        let config = Config::from_element(&element)?;
        Ok(config)
    }
}

fn main() {
    // Get the args supplied to the program
    let args = cli::Args::parse();
    // Run the program, printing out any errors
    if let Err(err) = run(args) {
        println!("{err}");
    }
}

fn run(args: cli::Args) -> Result<(), Error> {
    // Try to open the configuration file
    let config = Config::load(args.config)?;
    // Run the command
    match args.command {
        cli::Commands::Init => handle_init(config)?,
    }
    Ok(())
}

fn handle_init(config: Config) -> Result<(), Error> {
    for repo in config.repositories {
        // Ensure the repository exists
        repo.ensure_exists(&config.store)?;
        // Create all the symlinks
        for target in repo.symlinks(&config.root) {
            // Ensure the parent directory exists
            std::fs::create_dir_all(
                target.parent().unwrap(),
            )
            .unwrap();
            // Create the symlink
            // std::os::unix::fs::symlink(original, link)
            todo!()
        }
    }
    Ok(())
}
