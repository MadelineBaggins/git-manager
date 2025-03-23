// SPDX-FileCopyrightText: 2025 Madeline Baggins <declanbaggins@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::PathBuf, process::Command};

use maddi_xml::{Element, FromElement, Parser, Result};

#[derive(clap::Parser)]
struct Args {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(default_value = "")]
    search: String,
}

impl Args {
    pub fn config(&self) -> Option<PathBuf> {
        // Get the home directory
        let home: PathBuf =
            std::env::var("HOME").unwrap().into();
        let file = |path| {
            let file = home.join(path);
            file.exists().then_some(file)
        };
        self.config
            .clone()
            .or(file(".smartget.xml"))
            .or(file(".config/smartget.xml"))
            .or(file(".config/smartget/config.xml"))
            .or(file("/etc/smartget.xml"))
    }
}

struct Config {
    remotes: Vec<Box<dyn Remote>>,
}

trait Remote {
    fn search(&self, search: &str) -> Vec<String>;
}

struct Ssh {
    remote: Option<String>,
    command: String,
    args: Vec<String>,
}

impl<'a, 'b> FromElement<'a, 'b> for Ssh {
    fn from_element(
        element: &'b Element<'a>,
    ) -> Result<'a, Self> {
        Ok(Self {
            remote: element.optional_child("remote")?,
            command: element.child("command")?,
            args: element
                .children("arg")
                .collect::<Result<_>>()?,
        })
    }
}

impl Remote for Ssh {
    fn search(&self, search: &str) -> Vec<String> {
        let mut command = self.command.clone();
        for arg in &self.args {
            command.push(' ');
            command.push_str(arg);
        }
        command.push_str(" '");
        command.push_str(search);
        command.push('\'');
        String::from_utf8_lossy(
            &Command::new("ssh")
                .arg(self.remote.as_ref().unwrap())
                .arg("-t")
                .arg(command)
                .output()
                .unwrap()
                .stdout,
        )
        .lines()
        .map(|line| line.into())
        .collect()
    }
}

const RED: &str = "\x1b[1;31m";
const DEFAULT: &str = "\x1b[1;39m";

fn main() {
    // Get the arguments
    let args: Args = clap::Parser::parse();
    // Load the configuration file
    let Some(config_path) = args.config() else {
        println!(
            "{RED}error{DEFAULT}: could not find any configuration file"
        );
        return;
    };
    // Read in the configuration file
    let config_string =
        std::fs::read_to_string(&config_path).unwrap();
    // Parse the config
    let mut parser =
        Parser::new(&config_path, &config_string);
    let mut config = Config {
        remotes: Vec::new(),
    };
    while let Some(element) =
        parser.parse::<Option<Result<Element>>>()
    {
        match element {
            Ok(e) => match Ssh::from_element(&e) {
                Ok(s) => config.remotes.push(Box::new(s)),
                Err(e) => {
                    println!("{e}");
                    return;
                }
            },
            Err(e) => {
                println!("{e}");
                return;
            }
        }
    }
    // Search all the endpoints
    let results =
        config.remotes.iter().flat_map(move |remote| {
            remote.search(&args.search)
        });
    for result in results {
        println!("{result}");
    }
}
