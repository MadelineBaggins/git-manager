use std::{
    path::{Path, PathBuf},
    process::Command,
};

use maddi_xml::{self as xml, FromElement, Result};

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

impl Repository {
    pub fn ensure_exists(
        &self,
        store: &Path,
    ) -> std::result::Result<PathBuf, crate::Error> {
        // Check if the repository already exists
        let store_path = store.join(&self.name);
        if store_path.exists() {
            return Ok(store_path);
        }
        // Create the repository
        Command::new("git")
            .arg("init")
            .arg(&store_path)
            .output()
            .map_err(|_| {
                crate::Error::FailedToInitRepository(
                    store_path.clone(),
                )
            })?;
        // Configure the repository to accept pushes
        Command::new("git")
            .args([
                "config",
                "--local",
                "receive.denyCurrentBranch",
                "updateInstead",
            ])
            .current_dir(&store_path)
            .output()
            .map_err(|_| {
                crate::Error::FailedToConfigureRepository(
                    store_path.clone(),
                )
            })?;
        Ok(store_path)
    }

    pub fn symlinks<'a, 'b>(
        &'a self,
        symlinks_dir: &'b Path,
    ) -> impl Iterator<Item = PathBuf> + use<'a, 'b> {
        self.symlinks
            .iter()
            .map(|s| symlinks_dir.join(&s.path))
    }
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
    pub store: PathBuf,
    pub symlinks: PathBuf,
    pub repositories: Vec<Repository>,
}

impl<'a, 'b> FromElement<'a, 'b> for Config {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> Result<'a, Self> {
        Ok(Self {
            store: element.child("store")?,
            symlinks: element.child("symlinks")?,
            repositories: element
                .children::<Repository>("repo")
                .collect::<Result<_>>()?,
        })
    }
}
