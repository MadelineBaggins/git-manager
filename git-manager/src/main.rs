use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use cfg::{Config, Repository};
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
        Init(InitArgs),
        Switch,
    }

    #[derive(clap::Args)]
    pub struct InitArgs {
        #[arg(long)]
        pub symlinks: std::path::PathBuf,
        #[arg(long)]
        pub store: std::path::PathBuf,
    }
}

const RED: &str = "\x1b[1;31m";
const DEFAULT: &str = "\x1b[1;39m";

pub enum Error {
    MaddiXml(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
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
            Error::IoError(err) => {
                writeln!(
                    f,
                    "{RED}Io Error:{DEFAULT}\n{err:?}"
                )
            }
            Error::MaddiXml(raw) => write!(f, "{raw}"),
        }
    }
}

impl Config {
    fn load(path: PathBuf) -> Result<Self, Error> {
        // Open the configuration file
        let mut file = File::open(&path)?;
        // Read in the configuration file
        let mut source = String::new();
        file.read_to_string(&mut source)?;
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
        cli::Commands::Init(init_args) => {
            handle_init(init_args)?
        }
        cli::Commands::Switch => handle_switch(args)?,
    }
    Ok(())
}

fn handle_init(args: cli::InitArgs) -> Result<(), Error> {
    // Create the store
    std::fs::create_dir_all(&args.store)?;
    // create the symlinks dir
    std::fs::create_dir_all(&args.symlinks)?;
    // Build the configuration file
    let config = include_str!("config.xml")
        .replace(
            "$SYMLINKS",
            args.symlinks.to_str().unwrap(),
        )
        .replace("$STORE", args.store.to_str().unwrap());
    // Initialize the admin repository
    let admin = Repository::admin()
        .switch(&args.symlinks, &args.store)?;
    // Write the example configuration file
    std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(admin.join("config.xml"))
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
        repo.switch(&config.symlinks, &config.store)?;
    }
    Ok(())
}
