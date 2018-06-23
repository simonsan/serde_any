use std::fmt;
use std::ffi::OsStr;
use std::path::Path;

/// Serialization or deserialization formats
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// TOML (Tom's Obvious, Minimal Language), enabled by the `toml` feature.
    Toml,
    /// JSON (JavaScript Object Notation), enabled by the `json` feature.
    Json,
    /// YAML (YAML Ain't Markup Language), enabled by the `yaml` feature.
    Yaml,
    /// RON (Rusty Object Notation), enabled by the `ron` feature.
    Ron,
}

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

    f
}

/// Return a list of recognized file extensions
///
/// The return value depends on the features used when building serde_any.
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
    {
        e.push("ron");
    }

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
        ];
        for (f, n) in formats {
            let d = format!("{}", f);
            assert_eq!(&d, n);
        }
    }
}
