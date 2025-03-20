// SPDX-FileCopyrightText: 2025 Madeline Baggins <declanbaggins@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

#[derive(clap::Parser)]
struct Args {
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() {
    // Get the arguments
    let args: Args = clap::Parser::parse();
    // Load the configuration file
    println!("Hello, world!");
}
