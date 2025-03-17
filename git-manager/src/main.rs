use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::Command,
};

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
        Switch,
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
    FailedToCreateSymlink(PathBuf, PathBuf),
    ConfigExists(PathBuf),
    FailedToFindEnvVar(&'static str),
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
            Error::FailedToCreateSymlink(
                source,
                target,
            ) => {
                writeln!(f, "{RED}Error:{DEFAULT}")?;
                write!(
                    f,
                    "failed to create symlink '{} -> {}'",
                    source.display(),
                    target.display()
                )
            }
            Error::ConfigExists(path) => {
                writeln!(f, "{RED}Error:{DEFAULT}")?;
                let path = path.display();
                write!(f, "config already exists at '{path}', could not initialize")
            }
            Error::FailedToFindEnvVar(var) => {
                writeln!(f, "{RED}Error:{DEFAULT}")?;
                write!(f, "failed to get env var '{var}'")
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
    // Run the command
    match args.command {
        cli::Commands::Init => handle_init(args)?,
        cli::Commands::Switch => handle_switch(args)?,
    }
    Ok(())
}

fn handle_init(args: cli::Args) -> Result<(), Error> {
    // Ensure the config file doesn't already exist
    if args.config.exists() {
        return Err(Error::ConfigExists(args.config));
    }
    // Build the configuration file
    let home = std::env::var_os("HOME")
        .ok_or(Error::FailedToFindEnvVar("HOME"))?;
    let config = include_str!("../config.xml")
        .replace("$HOME", home.to_str().unwrap());
    // Write the example configuration file
    std::fs::File::options()
        .create_new(true)
        .open(args.config)
        .unwrap()
        .write_all(config.as_bytes())
        .unwrap();
    Ok(())
}

fn handle_switch(args: cli::Args) -> Result<(), Error> {
    // Try to open the configuration file
    let config = Config::load(args.config)?;
    // Print the configuration
    println!("{config:#?}");
    // Reconfigure everything to match the config
    for repo in config.repositories {
        // Ensure the repository exists
        let path = repo.ensure_exists(&config.store)?;
        // Create all the symlinks
        for target in repo.symlinks(&config.symlinks) {
            // Ensure the parent directory exists
            std::fs::create_dir_all(
                target.parent().unwrap(),
            )
            .unwrap();
            // Create the symlink
            Command::new("ln")
                .arg("-s")
                .arg(&path)
                .arg(&target)
                .output()
                .map_err(|_| {
                    Error::FailedToCreateSymlink(
                        path.clone(),
                        target.clone(),
                    )
                })?;
        }
    }
    Ok(())
}
