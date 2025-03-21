// SPDX-FileCopyrightText: 2025 Madeline Baggins <declanbaggins@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use maddi_xml as xml;

use clap::Parser as _;
use error::*;
use xml::FromElement as _;

mod cfg;
mod cli;
mod error;

impl cfg::Config {
    fn load(path: &Path) -> Result<Self, Error> {
        // Open the configuration file
        let mut file = File::open(path).with(path)?;
        // Read in the configuration file
        let mut source = String::new();
        file.read_to_string(&mut source).with(path)?;
        // Create the parser
        let mut parser = xml::Parser::new(path, &source);
        // Get the first piece of content in the file
        let content =
            parser
                .parse::<Option<
                    Result<xml::Content, maddi_xml::Error>,
                >>()
                .transpose()?;
        // Ensure the content was an element named 'config'
        let element = match content {
            Some(xml::Content::Element(e)) => {
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
        let config = cfg::Config::from_element(&element)?;
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
        cli::Commands::Init {
            command: cli::InitCommands::Server(init_args),
        } => handle_init(init_args)?,
        cli::Commands::Switch => handle_switch(args)?,
        cli::Commands::Search { ref search } => {
            handle_search(&args, search)?
        }
    }
    Ok(())
}

fn handle_init(
    args: cli::InitServerArgs,
) -> Result<(), Error> {
    // Create the store
    std::fs::create_dir_all(&args.store)
        .with(args.store.as_path())?;
    // create the symlinks dir
    std::fs::create_dir_all(&args.symlinks)
        .with(args.store.as_path())?;
    // Build the configuration file
    let config = include_str!("config.xml")
        .replace(
            "$SYMLINKS",
            args.symlinks.to_str().unwrap(),
        )
        .replace("$BRANCH", &args.branch)
        .replace("$STORE", args.store.to_str().unwrap());
    // Initialize the admin repository
    let admin = cfg::Repository::admin().switch(
        &args.branch,
        &args.symlinks,
        &args.store,
    )?;
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

fn handle_search(
    args: &cli::Args,
    search: &str,
) -> Result<(), Error> {
    // Try to open the configuration file
    let config = cfg::Config::load(&args.config)?;
    // Print all the results out to stdout
    let results = config.repositories.iter().filter_map(
        |repository| {
            repository
                .smartget_filter_map(search, &config.store)
        },
    );
    for result in results {
        println!("{}", result);
    }
    Ok(())
}

fn handle_switch(args: cli::Args) -> Result<(), Error> {
    // Try to open the configuration file
    let config = cfg::Config::load(&args.config)?;
    // Reconfigure everything to match the config
    for repo in config.repositories {
        // Ensure the repository exists
        repo.switch(
            &config.branch,
            &config.symlinks,
            &config.store,
        )?;
    }
    Ok(())
}
