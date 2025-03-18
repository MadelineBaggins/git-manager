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

#[derive(Debug)]
pub struct Repository {
    name: String,
    symlinks: Vec<Symlink>,
    hooks: Hooks,
}

impl Repository {
    pub fn ensure_correct(
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
        // Ensure the repositories hooks are correct
        self.hooks
            .update(&store_path.join(".git/hooks"))
            .map_err(|_| {
            crate::Error::FailedToConfigureHooks(
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

impl<'a, 'b> xml::FromElement<'a, 'b> for Repository {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> xml::Result<'a, Self> {
        Ok(Self {
            name: element.attribute::<&str>("name")?.into(),
            symlinks: element
                .children::<Symlink>("symlink")
                .collect::<xml::Result<_>>()?,
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
