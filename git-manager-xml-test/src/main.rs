use std::{io::Read, path::PathBuf};

use git_manager_xml::{Content, Error, Parser};
fn main() -> Result<(), std::io::Error> {
    // Get the target file
    let Some(target) = std::env::args().nth(1) else {
        return Ok(());
    };
    // Read in the target file
    let mut xml_src = String::new();
    std::fs::File::open(&target)
        .unwrap()
        .read_to_string(&mut xml_src)
        .unwrap();
    // Parse the xml
    let mut parser =
        Parser::new(PathBuf::from(target), &xml_src);
    let xml =
        parser.parse::<Option<Result<Content, Error>>>();
    // Print out the result
    match xml {
        None => println!("No content found..."),
        Some(Ok(content)) => println!("{content:#?}"),
        Some(Err(err)) => println!("{err}"),
    }
    Ok(())
}
