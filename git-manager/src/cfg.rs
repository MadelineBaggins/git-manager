// SPDX-FileCopyrightText: 2025 Madeline Baggins <declanbaggins@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use maddi_xml as xml;

#[derive(Debug)]
struct Symlink {
    path: PathBuf,
}

impl<'a, 'b> xml::FromElement<'a, 'b> for Symlink {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> xml::Result<'a, Self> {
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
pub enum HookKind {
    PreReceive,
    Update,
    PostReceive,
}

impl<'a, 'b> xml::FromValue<'a, 'b> for HookKind {
    fn from_value(
        value: &'b str,
        position: &'b maddi_xml::Position<'a>,
    ) -> xml::Result<'a, Self> {
        match value {
            "pre-receive" => Ok(HookKind::PreReceive),
            "update" => Ok(HookKind::Update),
            "post-receive" => Ok(HookKind::PostReceive),
            _ => Err(position.error("expected 'pre-receive', 'update', or 'post-receive'".into()))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Source {
    Inline(String),
    File(PathBuf),
}
impl<'a, 'b> xml::FromElement<'a, 'b> for Source {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> xml::Result<'a, Self> {
        const ERR: &str =
            "expected file content or 'src' attribute";
        let src =
            element.attribute::<Option<PathBuf>>("src")?;
        match (src, element.contents.as_slice()) {
            (Some(path), []) => Ok(Source::File(path)),
            (None, [xml::Content::Text(source)]) => {
                Ok(Source::Inline(source.into()))
            }
            _ => Err(element.position.error(ERR.into())),
        }
    }
}

impl Source {
    pub fn value(self) -> std::io::Result<String> {
        match self {
            Self::Inline(source) => Ok(source),
            Self::File(path) => {
                std::fs::read_to_string(&path)
            }
        }
    }
}

#[derive(Debug)]
pub struct Hooks {
    pre_receive: Option<Source>,
    update: Option<Source>,
    post_receive: Option<Source>,
}

impl Hooks {
    fn update_hook(
        path: &Path,
        source: &Option<Source>,
    ) -> std::io::Result<()> {
        // Delete the file
        if source.is_none() && path.exists() {
            return std::fs::remove_file(path);
        }
        // Create/Update the content of the file
        if let Some(content) =
            source.clone().map(Source::value).transpose()?
        {
            // Create/Update
            let mut file = std::fs::File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?;
            file.write_all(content.as_bytes())?;
            // Make the file executable
            Command::new("chmod")
                .arg("+x")
                .arg(path)
                .output()?;
        }
        Ok(())
    }
    pub fn update(
        &self,
        hook_directory: &Path,
    ) -> std::io::Result<()> {
        // Set up the pre_receive hook
        let pre_receive_file =
            hook_directory.join("pre-receive");
        Self::update_hook(
            &pre_receive_file,
            &self.pre_receive,
        )?;
        // Set up the update hook
        let update_file = hook_directory.join("update");
        Self::update_hook(&update_file, &self.update)?;
        // Set up the post_receive hook
        let post_receive_file =
            hook_directory.join("post-receive");
        Self::update_hook(
            &post_receive_file,
            &self.post_receive,
        )?;
        Ok(())
    }
}

impl<'a, 'b> xml::FromElement<'a, 'b> for Hooks {
    fn from_element(
        element: &'b maddi_xml::Element<'a>,
    ) -> maddi_xml::Result<'a, Self> {
        Ok(Self {
            pre_receive: element
                .optional_child("pre-receive")?,
            update: element.optional_child("update")?,
            post_receive: element
                .optional_child("post-receive")?,
        })
    }
}

struct Tag(String);

impl<'a, 'b> xml::FromElement<'a, 'b> for Tag {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> xml::Result<'a, Self> {
        match element.contents.as_slice() {
            [ maddi_xml::Content::Text(tag) ] => {
                if tag.contains(char::is_whitespace) {
                    Err(element.position.error("tag must not contain whitespace".into()))
                } else {
                    Ok(Tag(tag.into()))
                }
            },
            _ => Err(element.position.error("expected tag to contain text with no whitespace".into())),
        }
    }
}

#[derive(Debug)]
pub struct Repository {
    name: String,
    symlinks: Vec<Symlink>,
    tags: Vec<String>,
    hooks: Hooks,
}

impl Repository {
    pub fn smartget_filter_map(
        &self,
        search: &str,
        store_dir: &Path,
    ) -> Option<String> {
        let matches =
            search.split_whitespace().all(|term| {
                self.tags
                    .iter()
                    .any(|tag| tag.contains(term))
            }) || self.name.contains(search);
        if !matches {
            return None;
        }
        let store = store_dir.display();
        let name = &self.name;
        Some(format!("{name},git+ssh://{store}/{name}"))
    }
    pub fn admin() -> Self {
        Repository {
            name: "admin".into(),
            symlinks: vec![Symlink {
                path: "admin".into(),
            }],
            tags: vec![],
            hooks: Hooks {
                pre_receive: None,
                update: None,
                post_receive: Some(Source::Inline(
                    include_str!("post-update.sh").into(),
                )),
            },
        }
    }
    pub fn switch(
        &self,
        symlinks_dir: &Path,
        store_dir: &Path,
    ) -> std::result::Result<PathBuf, crate::Error> {
        // Check if the repository already exists
        let repository_path = store_dir.join(&self.name);
        if !repository_path.exists() {
            // Create the repository
            Command::new("git")
                .arg("init")
                .arg(&repository_path)
                .output()?;
        }
        // Configure the repository to accept pushes
        Command::new("git")
            .args([
                "config",
                "--local",
                "receive.denyCurrentBranch",
                "ignore",
            ])
            .current_dir(&repository_path)
            .output()?;
        // Ensure the repositories hooks are correct
        self.hooks
            .update(&repository_path.join(".git/hooks"))?;
        // Create all the symlinks
        self.build_symlinks(
            &repository_path,
            symlinks_dir,
        )?;
        Ok(repository_path)
    }
    fn symlinks<'a, 'b>(
        &'a self,
        symlinks_dir: &'b Path,
    ) -> impl Iterator<Item = PathBuf> + use<'a, 'b> {
        self.symlinks
            .iter()
            .map(|s| symlinks_dir.join(&s.path))
    }
    fn build_symlinks(
        &self,
        repository_path: &Path,
        symlinks_dir: &Path,
    ) -> std::io::Result<()> {
        // Create all the symlinks
        for target in self.symlinks(symlinks_dir) {
            // Ensure the parent directory exists
            std::fs::create_dir_all(
                target.parent().unwrap(),
            )?;
            // If the symlink exists, delete it.
            if target.exists() {
                std::fs::remove_file(&target)?;
            }
            // Create the symlink
            std::os::unix::fs::symlink(
                repository_path,
                target,
            )?;
        }
        Ok(())
    }
}

impl<'a, 'b> xml::FromElement<'a, 'b> for Repository {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> xml::Result<'a, Self> {
        Ok(Self {
            name: element.attribute::<&str>("name")?.into(),
            symlinks: element
                .children::<Symlink>("symlink")
                .collect::<xml::Result<_>>()?,
            tags: element
                .children::<Tag>("tag")
                .map(|tag| tag.map(|tag| tag.0))
                .collect::<Result<_, _>>()?,
            hooks: Hooks::from_element(element)?,
        })
    }
}

#[derive(Debug)]
pub struct Config {
    pub store: PathBuf,
    pub symlinks: PathBuf,
    pub repositories: Vec<Repository>,
}

impl<'a, 'b> xml::FromElement<'a, 'b> for Config {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> xml::Result<'a, Self> {
        Ok(Self {
            store: element.child("store")?,
            symlinks: element.child("symlinks")?,
            repositories: element
                .children::<Repository>("repo")
                .collect::<xml::Result<_>>()?,
        })
    }
}
