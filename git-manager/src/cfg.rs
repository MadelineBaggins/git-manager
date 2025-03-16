use std::path::PathBuf;

use maddi_xml::{
    self as xml, Element, FromElement, Parse, Result,
};

#[derive(Debug)]
struct Symlink {
    path: PathBuf,
}

impl<'a, 'b> FromElement<'a, 'b> for Symlink {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> Result<'a, Self> {
        use xml::Content;
        match element.contents.as_slice() {
            [Content::Text(path)] => Ok(Self {
                path: PathBuf::from(path),
            }),
            _ => Err(element
                .position
                .error("provide a path to symlink".into())),
        }
    }
}

#[derive(Debug)]
pub struct Repository {
    name: String,
    symlinks: Vec<Symlink>,
}

impl<'a, 'b> FromElement<'a, 'b> for Repository {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> Result<'a, Self> {
        Ok(Self {
            name: element.attribute::<&str>("name")?.into(),
            symlinks: element
                .children::<Symlink>("symlink")
                .collect::<Result<_>>()?,
        })
    }
}

#[derive(Debug)]
pub struct Config {
    store: PathBuf,
    repositories: Vec<Repository>,
}

impl<'a, 'b> FromElement<'a, 'b> for Config {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> Result<'a, Self> {
        Ok(Self {
            store: element.child("store")?,
            repositories: element
                .children::<Repository>("repo")
                .collect::<Result<_>>()?,
        })
    }
}
