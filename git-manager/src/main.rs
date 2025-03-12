use std::path::PathBuf;

use git_manager_xml::Element;

mod cfg;

mod cli {
    use clap::Parser as _;
    use clap::{Command, CommandFactory};
    use clap_complete::{generate, Generator, Shell};

    #[derive(clap::Parser)]
    pub struct Args {
        pub config: Option<std::path::PathBuf>,
        #[command(subcommand)]
        pub command: Commands,
    }

    impl Args {
        const POST_UPDATE: Args = Args {
            config: None,
            command: Commands::PostUpdate,
        };
        pub fn get() -> Self {
            let name = std::env::args().next();
            match name.as_deref() {
                Some("post_update") => Self::POST_UPDATE,
                Some(cmd)
                    if cmd.ends_with("/post_update") =>
                {
                    Self::POST_UPDATE
                }
                _ => Args::parse(),
            }
        }
    }

    #[derive(clap::Subcommand)]
    pub enum Commands {
        PostUpdate,
        Completions {
            shell: Shell,
        },
        Server {
            #[command(subcommand)]
            subcommand: ServerCommand,
        },
    }

    impl Commands {
        pub fn exec(self) {
            match self {
                Commands::Completions { shell } => {
                    let mut cmd = Args::command();
                    print_completions(shell, &mut cmd)
                }
                Commands::PostUpdate => {
                    println!("Running 'post_update'...");
                }
                Commands::Server { subcommand } => {
                    subcommand.exec()
                }
            }
        }
    }

    fn print_completions<G: Generator>(
        generator: G,
        cmd: &mut Command,
    ) {
        generate(
            generator,
            cmd,
            cmd.get_name().to_string(),
            &mut std::io::stdout(),
        )
    }

    #[derive(clap::Subcommand)]
    pub enum ServerCommand {
        Init,
    }

    impl ServerCommand {
        fn exec(self) {
            match self {
                ServerCommand::Init => Self::init(),
            }
        }
        fn init() {
            use std::process::Command;
            // Create the
            Command::new("git")
                .args(["init", "by-id/admin"])
                .output()
                .expect("Could not run `git init admin`");
        }
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

fn main() {
    use cfg::FromElement;
    use git_manager_xml as xml;

    // cli::Args::get().command.exec();
    // Read the config
    let path =
        PathBuf::from(std::env::args().nth(1).unwrap());
    let config = std::fs::read_to_string(&path).unwrap();
    // Parse the config
    let mut parser = xml::Parser::new(&path, &config);
    let element = parser
        .parse::<Option<Result<Element, xml::Error>>>()
        .unwrap();
    let element = match element {
        Ok(e) => e,
        Err(e) => {
            println!("{e}");
            return;
        }
    };
    let config = match cfg::Config::from_element(&element) {
        Ok(config) => config,
        Err(e) => {
            println!("{e}");
            return;
        }
    };
    println!("{config:#?}")
}
