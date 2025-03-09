#[derive(Clone)]
pub struct Parser<'a> {
    src: &'a str,
    tail: &'a str,
    line: usize,
    char: usize,
}

#[derive(Debug)]
pub struct Error {
    pub src: String,
    pub message: String,
    pub line: usize,
    pub char: usize,
}

impl std::fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        const RED: &str = "\x1b[1;31m";
        const DEFAULT: &str = "\x1b[1;39m";
        writeln!(f, "{RED}Error:{DEFAULT}")?;
        for (line_num, line) in
            self.src.split('\n').enumerate()
        {
            writeln!(f, "{line}")?;
            if line_num == self.line {
                let offset =
                    std::iter::repeat_n(' ', self.char)
                        .collect::<String>();
                writeln!(f, "{offset}^")?;
                let offset_len = self
                    .char
                    .saturating_sub(self.message.len());
                let offset =
                    std::iter::repeat_n(' ', offset_len)
                        .collect::<String>();
                writeln!(
                    f,
                    "{offset}{RED}{}{DEFAULT}",
                    self.message
                )?;
            }
        }
        Ok(())
    }
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            tail: src,
            line: 0,
            char: 0,
        }
    }
    fn error(&mut self, message: String) -> Error {
        Error {
            src: self.src.into(),
            message,
            line: self.line,
            char: self.char,
        }
    }
    pub fn parse<T: Parse>(&mut self) -> T {
        T::parse(self)
    }
    fn take_whitespace(&mut self) {
        let len = self
            .tail
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(self.tail.len());
        self.take(len);
    }
    fn take_char(&mut self) -> Option<char> {
        let char = self.tail.chars().next()?;
        match char {
            '\n' => {
                self.line += 1;
                self.char = 0;
            }
            _ => self.char += 1,
        }
        (_, self.tail) =
            self.tail.split_at(char.len_utf8());
        Some(char)
    }
    fn take(&mut self, n: usize) -> &'a str {
        let head;
        (head, self.tail) = self.tail.split_at(n);
        for c in head.chars() {
            match c {
                '\n' => {
                    self.line += 1;
                    self.char = 0;
                }
                _ => self.char += 1,
            }
        }
        head
    }
}

pub trait Parse {
    fn parse(parser: &mut Parser) -> Self;
}

#[derive(Debug)]
pub enum Content {
    Element(Element),
    Text(String),
}

impl Parse for Option<Result<Content, Error>> {
    fn parse(parser: &mut Parser) -> Self {
        // Clear any whitespace
        parser.take_whitespace();
        // If the document has finished parsing
        if parser.tail.is_empty() {
            return None;
        };
        // Check if we start with an element
        match parser
            .parse::<Option<Result<Element, Error>>>()
        {
            Some(Ok(element)) => {
                return Some(Ok(Content::Element(element)))
            }
            Some(Err(err)) => return Some(Err(err)),
            None => {}
        }
        // Otherwise, get the text
        let len = parser
            .tail
            .find('<')
            .unwrap_or(parser.tail.len());
        let text = parser.take(len);
        Some(Ok(Content::Text(text.into())))
    }
}

#[derive(Debug)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub contents: Vec<Content>,
}

impl Parse for Option<Result<Element, Error>> {
    fn parse(parser: &mut Parser) -> Self {
        // Find the opening tag if there is one
        let open_tag = match parser
            .parse::<Option<Result<OpenTag, Error>>>()?
        {
            Ok(open_tag) => open_tag,
            Err(err) => return Some(Err(err)),
        };
        // If the tag was self closing, return the entity
        if open_tag.self_closing {
            return Some(Ok(Element {
                name: open_tag.name,
                attributes: open_tag.attributes,
                contents: vec![],
            }));
        }
        // Parse all the content
        let mut contents = vec![];
        let close_tag = loop {
            // Remove any whitespace
            parser.take_whitespace();
            // Check if there's a closing tag
            if let Some(close_tag) = parser
                .parse::<Option<Result<CloseTag, Error>>>()
            {
                break close_tag;
            }
            // Otherwise, try to get content
            match parser
                .parse::<Option<Result<Content, Error>>>()
            {
                Some(Err(err)) => return Some(Err(err)),
                Some(Ok(content)) => contents.push(content),
                None => {
                    return Some(Err(parser.error(
                        "missing closing tag".into(),
                    )))
                }
            }
        };
        // Ensure we didn't error getting the close tag
        let close_tag = match close_tag {
            Ok(close_tag) => close_tag,
            Err(err) => return Some(Err(err)),
        };
        // Ensure the close and open tags match
        if open_tag.name != close_tag.name {
            return Some(Err(parser
                .error("mismatched closing tag".into())));
        }
        Some(Ok(Element {
            name: open_tag.name,
            attributes: open_tag.attributes,
            contents,
        }))
    }
}

/// The name of an element.
/// - Must start with a letter or underscore.
/// - Cannot start with the letters "xml" in any case.
/// - Consists only of letters, digits, hyphens,
///   underscores, and periods.
struct Name(String);

impl Parse for Option<Name> {
    fn parse(parser: &mut Parser) -> Option<Name> {
        // Ensure tail starts with a letter or underscore
        if !parser.tail.starts_with(|c: char| {
            c.is_alphabetic() || c == '_'
        }) {
            return None;
        }
        // Ensure tail doesn't start with 'xml' in any case
        if parser
            .tail
            .get(0..3)
            .is_some_and(|f| f.to_lowercase() == "xml")
        {
            return None;
        }
        // Find the head of the tail that only consists of
        // digits, hyphens, underscores, and periods.
        let len = parser
            .tail
            .find(|c: char| {
                !c.is_ascii_alphanumeric()
                    && !['.', '_', '-'].contains(&c)
            })
            .unwrap_or(parser.tail.len());
        let name = parser.tail.get(..len).unwrap();
        (!name.is_empty())
            .then_some(Name(parser.take(len).into()))
    }
}

struct OpenTag {
    name: String,
    attributes: Vec<Attribute>,
    self_closing: bool,
}

impl Parse for Option<Result<OpenTag, Error>> {
    fn parse(parser: &mut Parser) -> Self {
        // Ensure we're parsing an open tag
        if !parser.tail.starts_with('<') {
            return None;
        }
        // Skip over the opening chevron
        parser.take(1);
        // Get the element's name
        let Some(Name(name)) =
            parser.parse::<Option<Name>>()
        else {
            return Some(Err(parser
                .error("expected element name".into())));
        };
        // Skip any whitespace
        parser.take_whitespace();
        // Parse any attributes
        let mut attributes = vec![];
        while let Some(attribute) = parser
            .parse::<Option<Result<Attribute, Error>>>()
        {
            match attribute {
                Ok(attribute) => attributes.push(attribute),
                Err(e) => return Some(Err(e)),
            }
            parser.take_whitespace();
        }
        // Ensure the opening tag ends with '/>' or '>'.
        let self_closing = parser.tail.starts_with("/>");
        if !self_closing && !parser.tail.starts_with(">") {
            return Some(Err(
                parser.error("expected '>' or '/>'".into())
            ));
        }
        // Skip the ending bit
        if self_closing {
            parser.take("/>".len());
        } else {
            parser.take(">".len());
        }
        // Build the opening tag
        Some(Ok(OpenTag {
            name,
            attributes,
            self_closing,
        }))
    }
}

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
}

impl Parse for Option<Result<Attribute, Error>> {
    fn parse(parser: &mut Parser) -> Self {
        // Clone the parser in case we need to restore it
        let backup = parser.clone();
        // Get the name of the attribute
        let Some(Name(name)) =
            parser.parse::<Option<Name>>()
        else {
            *parser = backup;
            return None;
        };
        // If there's no value to the attribute, finish
        // parsing.
        if !parser.tail.starts_with('=') {
            return Some(Ok(Attribute {
                name,
                value: None,
            }));
        }
        // Skip the '='
        parser.take(1);
        // Parse the value of the attribute
        let Some(AttributeValue(value)) =
            parser.parse::<Option<AttributeValue>>()
        else {
            return Some(Err(parser.error(
                "expected attribute value".into(),
            )));
        };
        Some(Ok(Attribute {
            name,
            value: Some(value),
        }))
    }
}

struct AttributeValue(String);

impl Parse for Option<AttributeValue> {
    fn parse(parser: &mut Parser) -> Self {
        // Ensure the parser starts with a single or double
        // quote.
        let quote = match parser.tail.chars().next()? {
            c @ ('"' | '\'') => c,
            _ => return None,
        };
        // Create a working copy of the parser
        let mut working = parser.clone();
        working.take(1);
        // Build out the string
        // TODO: Add support for character entities
        let mut value = String::new();
        loop {
            let next = working.take_char()?;
            match next {
                '\\' => match working.take_char()? {
                    c @ ('\\' | '\'' | '"') => {
                        value.push(c)
                    }
                    _ => return None,
                },
                c if c == quote => break,
                c => value.push(c),
            }
        }
        // Save the working copy of the parser
        *parser = working;
        Some(AttributeValue(value))
    }
}

struct CloseTag {
    name: String,
}

impl Parse for Option<Result<CloseTag, Error>> {
    fn parse(parser: &mut Parser) -> Self {
        // Ensure we're at the start of a closing tag
        if !parser.tail.starts_with("</") {
            return None;
        }
        parser.take("</".len());
        // Get the name of the closing tag
        let Some(Name(name)) =
            parser.parse::<Option<Name>>()
        else {
            return Some(Err(parser
                .error("expected element name".into())));
        };
        // Ensure we end with a '>'.
        if !parser.tail.starts_with('>') {
            return Some(Err(
                parser.error("expected '>'".into())
            ));
        }
        // Skip the '>'.
        parser.take(">".len());
        Some(Ok(CloseTag { name }))
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
