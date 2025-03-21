use maddi_xml as xml;

const RED: &str = "\x1b[1;31m";
const DEFAULT: &str = "\x1b[1;39m";

pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> From<xml::Error<'a>> for Error {
    fn from(value: xml::Error<'a>) -> Self {
        Self(format!("{value}"))
    }
}

impl<C: Context> From<With<std::io::Error, C>> for Error {
    fn from(
        With { inner, context }: With<std::io::Error, C>,
    ) -> Self {
        let kind = C::KIND;
        let context = context.display();
        Self(format!(
            "{RED}IO Error with {kind}{DEFAULT}\n\t{context}\n{RED}Error:{DEFAULT}\n{inner:?}"
        ))
    }
}

pub trait Context {
    const KIND: &'static str;
    fn display(self) -> impl std::fmt::Display;
}

impl Context for [&std::path::Path; 2] {
    const KIND: &'static str = "symlink";

    fn display(self) -> impl std::fmt::Display {
        format!(
            "{} -> {}",
            self[0].display(),
            self[1].display()
        )
    }
}

impl Context for &std::path::Path {
    const KIND: &'static str = "path";
    fn display(self) -> impl std::fmt::Display {
        self.display()
    }
}

impl Context for std::process::Command {
    const KIND: &'static str = "command";

    fn display(self) -> impl std::fmt::Display {
        format!("{self:?}")
    }
}

pub struct With<T, C: Context> {
    inner: T,
    context: C,
}

pub trait ResultExt<T> {
    fn with<C: Context>(
        self,
        context: C,
    ) -> Result<T, With<std::io::Error, C>>;
}

impl<T> ResultExt<T> for std::io::Result<T> {
    fn with<C: Context>(
        self,
        context: C,
    ) -> Result<T, With<std::io::Error, C>> {
        self.map_err(|err| With {
            inner: err,
            context,
        })
    }
}
