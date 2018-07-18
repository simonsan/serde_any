use std::fmt;
use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;

/// Serialization or deserialization formats
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// TOML (Tom's Obvious, Minimal Language), enabled by the `toml` feature, implemented using [`toml`](https://docs.rs/toml).
    Toml,
    /// JSON (JavaScript Object Notation), enabled by the `json` feature, implemented using [`serde_json`](https://docs.rs/serde_json).
    Json,
    /// YAML (YAML Ain't Markup Language), enabled by the `yaml` feature, implemented using [`serde_yaml`](https://docs.rs/serde_yaml).
    Yaml,
    /// RON (Rusty Object Notation), enabled by the `ron` feature, implemented using [`ron`](https://docs.rs/ron).
    Ron,
    /// XML (Rusty Object Notation), enabled by the `xml` feature, implemented using [`serde-xml-rs`](https://docs.rs/serde-xml-rs).
    Xml,
    /// Url encoding (also known as percent encoding), enabled by the `url` feature, implemented using [`serde_urlencode`](https://docs.rs/serde_urlencode).
    Url,
}

/// The common error type
#[derive(Debug, Fail)]
#[fail(display = "Unknown format name {}", _0)]
pub struct UnknownFormatStringError(String);

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Format {
    /// Checks whether this format is supported
    ///
    /// Support for different formats is controlled by the features used
    /// when building `serde_any`.
    pub fn is_supported(&self) -> bool {
        match self {
            Format::Toml => cfg!(feature = "toml"),
            Format::Json => cfg!(feature = "json"),
            Format::Yaml => cfg!(feature = "yaml"),
            Format::Ron => cfg!(feature = "ron"),
            Format::Xml => cfg!(feature = "xml"),
            Format::Url => cfg!(feature = "url"),
        }
    }
}

impl FromStr for Format {
    type Err = UnknownFormatStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "toml" => Ok(Format::Toml),
            "json" => Ok(Format::Json),
            "yaml" => Ok(Format::Yaml),
            "ron" => Ok(Format::Ron),
            "xml" => Ok(Format::Xml),
            "url" => Ok(Format::Url),
            s => Err(UnknownFormatStringError(s.to_string())),
        }
    }
}

/// Return a list of supported formats
///
/// Support for different formats is controlled by the features used
/// when building `serde_any`.
pub fn supported_formats() -> Vec<Format> {
    let mut f = Vec::new();

    #[cfg(feature = "toml")]
    f.push(Format::Toml);

    #[cfg(feature = "json")]
    f.push(Format::Json);

    #[cfg(feature = "yaml")]
    f.push(Format::Yaml);

    #[cfg(feature = "ron")]
    f.push(Format::Ron);

    #[cfg(feature = "xml")]
    f.push(Format::Xml);

    #[cfg(feature = "url")]
    f.push(Format::Url);

    f
}

/// Return a list of recognized file extensions
///
/// The return value depends on the features used when building `serde_any`.
/// Only file extensions corresponding to supported formats will be returned.
pub fn supported_extensions() -> Vec<&'static str> {
    let mut e = Vec::new();

    #[cfg(feature = "toml")]
    e.push("toml");

    #[cfg(feature = "json")]
    e.push("json");

    #[cfg(feature = "yaml")]
    {
        e.push("yml");
        e.push("yaml");
    }

    #[cfg(feature = "ron")]
    e.push("ron");

    #[cfg(feature = "xml")]
    e.push("xml");

    e
}

/// Attempt to guess the serialization/deserialization format from a file name
///
/// This function may recognize and return a format even if it's not supported due to feature flags.
pub fn guess_format<P>(path: P) -> Option<Format>
where
    P: AsRef<Path>,
{
    path.as_ref()
        .extension()
        .and_then(OsStr::to_str)
        .and_then(guess_format_from_extension)
}

/// Attempt to guess the serialization/deserialization format from a file extension
///
/// This function may recognize and return a format even if it's not supported due to feature flags.
pub fn guess_format_from_extension(ext: &str) -> Option<Format> {
    match ext {
        "yml" | "yaml" => Some(Format::Yaml),
        "json" => Some(Format::Json),
        "toml" => Some(Format::Toml),
        "ron" => Some(Format::Ron),
        "xml" => Some(Format::Xml),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn extensions() {
        for ext in supported_extensions() {
            let stem = Path::new("test");
            let from_ext = guess_format_from_extension(ext);
            let from_path = guess_format(stem.with_extension(ext));
            assert!(from_ext.is_some());
            assert!(from_path.is_some());
            assert_eq!(from_ext, from_path);
        }
    }

    #[test]
    fn display_format() {
        let formats = vec![
            (Format::Json, "Json"),
            (Format::Toml, "Toml"),
            (Format::Yaml, "Yaml"),
            (Format::Ron, "Ron"),
            (Format::Xml, "Xml"),
            (Format::Url, "Url"),
        ];
        for (f, n) in formats {
            let d = format!("{}", f);
            assert_eq!(&d, n);
        }
    }

    #[test]
    fn parse_format() {
        let formats = vec![
            (Format::Json, "Json"),
            (Format::Toml, "Toml"),
            (Format::Yaml, "Yaml"),
            (Format::Ron, "Ron"),
            (Format::Xml, "Xml"),
            (Format::Url, "Url"),
        ];
        for (f, n) in formats {
            let parsed_format: Format = n.parse().unwrap();
            assert_eq!(parsed_format, f);
        }
    }

    #[test]
    fn parse_format_invalid() {
        let invalid_format_strings = vec!["", "j", "a", "hobbit", "josn", "yoml", "yml"];
        for s in invalid_format_strings {
            let p = s.parse::<Format>();
            assert!(p.is_err());
        }
    }
}
