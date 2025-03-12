use std::path::PathBuf;

use git_manager_xml as xml;

type Result<'err, T> =
    std::result::Result<T, xml::Error<'err>>;

trait FromValue<'a, 'b>: Sized {
    fn from_value(
        value: &'b str,
        position: &'b xml::Position<'a>,
    ) -> Result<'a, Self>;
}

struct Int<T>(T);

impl<'a, 'b> FromValue<'a, 'b> for &'b str {
    fn from_value(
        value: &'b str,
        _position: &'b git_manager_xml::Position<'a>,
    ) -> Result<'a, Self> {
        Ok(value)
    }
}

impl<'a, 'b, T> FromValue<'a, 'b> for Int<T>
where
    T: std::str::FromStr<Err = std::num::ParseIntError>,
{
    fn from_value(
        value: &'b str,
        position: &'b git_manager_xml::Position<'a>,
    ) -> Result<'a, Self> {
        value.parse::<T>().map_err(|e| {
            let msg = match e.kind() {
                std::num::IntErrorKind::Empty =>
                "failed to parse integer from empty string",
                std::num::IntErrorKind::InvalidDigit => "value contains invalid digit",
                std::num::IntErrorKind::PosOverflow => "value too large for this attribute",
                std::num::IntErrorKind::NegOverflow => "value too small for this attribute",
                std::num::IntErrorKind::Zero => "value cannot be zero for this attribute",
                _ => "unknown integer parse error",
            }.to_string();
            position
                .error(msg)
        }).map(Int)
    }
}

trait FromAttribute<'a, 'b>: Sized {
    fn from_attribute(
        attribute: &'b xml::Attribute<'a>,
    ) -> Result<'a, Self>;
}

impl<'a, 'b, T: FromValue<'a, 'b>> FromAttribute<'a, 'b>
    for T
{
    fn from_attribute(
        attribute: &'b git_manager_xml::Attribute<'a>,
    ) -> Result<'a, Self> {
        let Some(value) = attribute.value.as_ref() else {
            let name = attribute.name;
            return Err(attribute.position.error(format!(
                "expected non-empty value for '{name}'"
            )));
        };
        T::from_value(value, &attribute.position)
    }
}

trait Query<'a, 'b>: Sized {
    fn get(
        name: &str,
        element: &'b xml::Element<'a>,
    ) -> Result<'a, Self>;
}

impl<'a, 'b, T: FromAttribute<'a, 'b>> Query<'a, 'b> for T {
    fn get(
        name: &str,
        element: &'b git_manager_xml::Element<'a>,
    ) -> Result<'a, Self> {
        let Some(attribute) = element.attributes.get(name)
        else {
            let msg =
                format!("expected '{name}' attribute");
            return Err(element.position.error(msg));
        };
        T::from_attribute(attribute)
    }
}

impl<'a, 'b, T: FromAttribute<'a, 'b>> Query<'a, 'b>
    for Option<T>
{
    fn get(
        name: &str,
        element: &'b git_manager_xml::Element<'a>,
    ) -> Result<'a, Self> {
        element
            .attributes
            .get(name)
            .map(|a| T::from_attribute(a))
            .transpose()
    }
}

impl<'a, 'b> Query<'a, 'b> for bool {
    fn get(
        name: &str,
        element: &'b git_manager_xml::Element<'a>,
    ) -> Result<'a, Self> {
        Ok(element.attributes.contains_key(name))
    }
}

trait ElementExt<'a> {
    fn get<'b, T: Query<'a, 'b>>(
        &'b self,
        name: &str,
    ) -> Result<'a, T>;
    fn children<'b, T: FromElement<'a, 'b>>(
        &'b self,
        name: &str,
    ) -> impl Iterator<Item = Result<'a, T>>;
}

impl<'a> ElementExt<'a> for xml::Element<'a> {
    fn get<'b, T: Query<'a, 'b>>(
        &'b self,
        name: &str,
    ) -> Result<'a, T> {
        T::get(name, self)
    }

    fn children<'b, T: FromElement<'a, 'b>>(
        &'b self,
        name: &str,
    ) -> impl Iterator<Item = Result<'a, T>> {
        use xml::Content;
        self.contents
            .iter()
            .filter_map(move |item| match item {
                Content::Element(e) if e.name == name => {
                    Some(e)
                }
                _ => None,
            })
            .map(|t| T::from_element(t))
    }
}

pub trait FromElement<'a, 'b>: Sized {
    fn from_element(
        element: &'b xml::Element<'a>,
    ) -> Result<'a, Self>;
}

#[derive(Debug)]
struct Symlink {
    path: PathBuf,
}

impl<'a, 'b> FromElement<'a, 'b> for Symlink {
    fn from_element(
        element: &'b git_manager_xml::Element<'a>,
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
    public: bool,
    symlinks: Vec<Symlink>,
}

impl<'a, 'b> FromElement<'a, 'b> for Repository {
    fn from_element(
        element: &'b git_manager_xml::Element<'a>,
    ) -> Result<'a, Self> {
        Ok(Self {
            name: element.get::<&str>("name")?.into(),
            public: element.get("public")?,
            symlinks: element
                .children::<Symlink>("symlink")
                .collect::<Result<_>>()?,
        })
    }
}

#[derive(Debug)]
pub struct Config {
    repositories: Vec<Repository>,
}

impl<'a, 'b> FromElement<'a, 'b> for Config {
    fn from_element(
        element: &'b git_manager_xml::Element<'a>,
    ) -> Result<'a, Self> {
        Ok(Self {
            repositories: element
                .children::<Repository>("repo")
                .collect::<Result<_>>()?,
        })
    }
}
