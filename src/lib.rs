#![warn(missing_docs)]

//! # Serde Any
//!
//! Dynamic serialization and deserialization with the format chosen at runtime
//!

#[macro_use]
extern crate failure;
extern crate serde;

#[cfg(feature = "toml")]
extern crate toml;

#[cfg(feature = "json")]
extern crate serde_json;

#[cfg(feature = "yaml")]
extern crate serde_yaml;

#[cfg(feature = "ron")]
extern crate ron;

use std::path::Path;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::fs::File;
use std::fmt;

use serde::de::{Deserialize, DeserializeOwned};
use serde::ser::Serialize;

/// The common error type
#[derive(Debug, Fail)]
pub enum Error {
    /// Error serializing or deserializing with JSON
    #[cfg(feature = "json")]
    #[fail(display = "JSON error: {}", _0)]
    Json(#[fail(cause)] serde_json::Error),

    /// Error serializing or deserializing with YAML
    #[cfg(feature = "yaml")]
    #[fail(display = "YAML error: {}", _0)]
    Yaml(#[fail(cause)] serde_yaml::Error),

    /// Error deserializing with TOML
    #[cfg(feature = "toml")]
    #[fail(display = "TOML deserialize error: {}", _0)]
    TomlDeserialize(#[fail(cause)] toml::de::Error),

    /// Error serializing with TOML
    #[cfg(feature = "toml")]
    #[fail(display = "TOML serialize error: {}", _0)]
    TomlSerialize(#[fail(cause)] toml::ser::Error),

    /// Error deserializing with RON
    #[cfg(feature = "ron")]
    #[fail(display = "RON deserialize error: {}", _0)]
    RonDeserialize(#[fail(cause)] ron::de::Error),

    /// Error serializing with RON
    #[cfg(feature = "ron")]
    #[fail(display = "RON serialize error: {}", _0)]
    RonSerialize(#[fail(cause)] ron::ser::Error),

    /// IO error
    #[fail(display = "IO error: {}", _0)]
    Io(#[fail(cause)] std::io::Error),

    /// The specified format is not supported
    #[fail(display = "Format {} not supported", _0)]
    UnsupportedFormat(Format),

    /// The specified file extension is not supported
    #[fail(display = "File extension {} not supported", _0)]
    UnsupportedFileExtension(String),

    /// None of the supported formats was able to deserialize successfully
    #[fail(display = "No format was able to parse the source")]
    NoSuccessfulParse,
}

macro_rules! impl_error_from {
    ($error_type:ty => $variant:expr) => (
        impl From<$error_type> for Error {
            fn from(e: $error_type) -> Error {
                $variant(e)
            }
        }
    );
}

impl_error_from!(std::io::Error => Error::Io);

#[cfg(feature = "json")]
impl_error_from!(serde_json::Error => Error::Json);

#[cfg(feature = "yaml")]
impl_error_from!(serde_yaml::Error => Error::Yaml);

#[cfg(feature = "toml")]
impl_error_from!(toml::ser::Error => Error::TomlSerialize);
#[cfg(feature = "toml")]
impl_error_from!(toml::de::Error => Error::TomlDeserialize);

#[cfg(feature = "ron")]
impl_error_from!(ron::ser::Error => Error::RonSerialize);
#[cfg(feature = "ron")]
impl_error_from!(ron::de::Error => Error::RonDeserialize);

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
    /// when building serde_any.
    pub fn is_supported(&self) -> bool {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "toml")]
            Format::Toml => true,
            #[cfg(feature = "json")]
            Format::Json => true,
            #[cfg(feature = "yaml")]
            Format::Yaml => true,
            #[cfg(feature = "ron")]
            Format::Ron => true,

            _ => false,
        }
    }
}

/// Return a list of supported formats
///
/// Support for different formats is controlled by the features used
/// when building serde_any.
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

/// Deserialize from an IO stream using a specified format
///
/// # Errors
///
/// If the specified format is not supported, this function returns
/// `Error::UnsupportedFormat`.
///
/// If the conversion itself fails, the format-specific variant of `Error`
/// will be returned, with the underlying error as its cause.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
/// extern crate failure;
///
/// use failure::Error;
/// use std::fs::File;
/// use std::path::Path;
///
/// use serde_any::Format;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn read_user_from_file<P: AsRef<Path>>(path: P, format: Format) -> Result<User, Error> {
///     // Open the file in read-only mode.
///     let file = File::open(path)?;
///
///     // Read the contents of the file as an instance of `User`.
///     let u = serde_any::from_reader(file, format)?;
///
///     // Return the `User`.
///     Ok(u)
/// }
///
/// fn main() {
///     match read_user_from_file("test.json", Format::Json) {
///         Ok(u) => println!("{:#?}", u),
///         Err(e) => println!("Error deserializing user: {}", e),
///     };
/// }
/// ```
#[allow(unreachable_patterns, unused_mut)]
pub fn from_reader<T, R>(mut reader: R, format: Format) -> Result<T, Error>
where
    T: DeserializeOwned,
    R: Read,
{
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => Ok(serde_yaml::from_reader::<_, T>(reader)?),
        #[cfg(feature = "json")]
        Format::Json => Ok(serde_json::from_reader::<_, T>(reader)?),
        #[cfg(feature = "toml")]
        Format::Toml => {
            let mut s = Vec::new();
            reader.read_to_end(&mut s)?;
            Ok(toml::from_slice::<T>(&s)?)
        }
        #[cfg(feature = "ron")]
        Format::Ron => Ok(ron::de::from_reader::<_, T>(reader)?),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}


/// Deserialize from a string using a specified format
///
/// # Errors
///
/// If the specified format is not supported, this function returns
/// `Error::UnsupportedFormat`.
///
/// If the conversion itself fails, the format-specific variant of `Error`
/// will be returned, with the underlying error as its cause.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
/// extern crate failure;
///
/// use failure::Error;
///
/// use serde_any::Format;
///
/// #[derive(Deserialize, Debug)]
/// struct Person {
///     name: String,
///     knowledge: u32,
/// }
///
/// fn main() -> Result<(), Error> {
///     let data = "{
///         \"name\": \"Jon Snow\",
///         \"knowledge\": 0
///     }";
///     let person: Person = serde_any::from_str(data, Format::Json)?;
///     println!("{:#?}", person);
///     Ok(())
/// }
/// ```
pub fn from_str<'a, T>(s: &'a str, format: Format) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    #[allow(unreachable_patterns)]
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => Ok(serde_yaml::from_str::<T>(s)?),
        #[cfg(feature = "json")]
        Format::Json => Ok(serde_json::from_str::<T>(s)?),
        #[cfg(feature = "toml")]
        Format::Toml => Ok(toml::from_str::<T>(s)?),
        #[cfg(feature = "ron")]
        Format::Ron => Ok(ron::de::from_str::<T>(s)?),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

/// Deserialize from a string using any supported format
///
/// This function will attempt to deserialize the string using each supported format,
/// and will return the result of the first successful deserialization.
///
/// # Errors
///
/// If none of the supported formats can deserialize the string successfully,
/// `Error::NoSuccessfulParse` is returned.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
/// extern crate failure;
///
/// use failure::Error;
///
/// use serde_any::Format;
///
/// #[derive(Deserialize, Debug)]
/// struct Person {
///     name: String,
///     knowledge: u32,
/// }
///
/// fn main() -> Result<(), Error> {
///     let data = "{
///         \"name\": \"Jon Snow\",
///         \"knowledge\": 0
///     }";
///     let person: Person = serde_any::from_str_any(data)?;
///     println!("{:#?}", person);
///     Ok(())
/// }
/// ```
pub fn from_str_any<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    for format in supported_formats() {
        match from_str(&s, format) {
            Ok(t) => return Ok(t),
            Err(_) => continue,
        }
    }

    Err(Error::NoSuccessfulParse)
}

/// Deserialize from a byte slice using a specified format
///
/// This function will attempt to deserialize the string using each supported format,
/// and will return the result of the first successful deserialization.
///
/// # Errors
///
/// If none of the supported formats can deserialize the string successfully,
/// `Error::NoSuccessfulParse` is returned.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
/// extern crate failure;
///
/// use failure::Error;
///
/// use serde_any::Format;
///
/// #[derive(Deserialize, Debug)]
/// struct Person {
///     name: String,
///     knowledge: u32,
/// }
///
/// fn main() -> Result<(), Error> {
///     let data = b"{
///         \"name\": \"Jon Snow\",
///         \"knowledge\": 0
///     }";
///     let person: Person = serde_any::from_slice(data, Format::Json)?;
///     println!("{:#?}", person);
///     Ok(())
/// }
/// ```
pub fn from_slice<'a, T>(s: &'a [u8], format: Format) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    #[allow(unreachable_patterns)]
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => Ok(serde_yaml::from_slice::<T>(s)?),
        #[cfg(feature = "json")]
        Format::Json => Ok(serde_json::from_slice::<T>(s)?),
        #[cfg(feature = "toml")]
        Format::Toml => Ok(toml::from_slice::<T>(s)?),
        #[cfg(feature = "ron")]
        Format::Ron => Ok(ron::de::from_bytes::<T>(s)?),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

/// Deserialize from a byte slice using any supported format
///
/// # Errors
///
/// If the specified format is not supported, this function returns
/// `Error::UnsupportedFormat`.
///
/// If the conversion itself fails, the format-specific variant of `Error`
/// will be returned, with the underlying error as its cause.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
/// extern crate failure;
///
/// use failure::Error;
///
/// use serde_any::Format;
///
/// #[derive(Deserialize, Debug)]
/// struct Person {
///     name: String,
///     knowledge: u32,
/// }
///
/// fn main() -> Result<(), Error> {
///     let data = b"{
///         \"name\": \"Jon Snow\",
///         \"knowledge\": 0
///     }";
///     let person: Person = serde_any::from_slice_any(data)?;
///     println!("{:#?}", person);
///     Ok(())
/// }
/// ```
pub fn from_slice_any<'a, T>(s: &'a [u8]) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    for format in supported_formats() {
        match from_slice(&s, format) {
            Ok(t) => return Ok(t),
            Err(_) => continue,
        }
    }

    Err(Error::NoSuccessfulParse)
}

pub fn from_file<T, P>(path: P) -> Result<T, Error>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let format = guess_format(&path);

    match format {
        Some(format) => from_reader(File::open(path)?, format),
        None => {
            let mut s = Vec::new();
            let mut reader = File::open(&path)?;
            reader.read_to_end(&mut s)?;

            Ok(from_slice_any(&s)?)
        }
    }
}

pub fn from_file_stem<T, P>(stem: P) -> Result<T, Error>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    for extension in supported_extensions() {
        let path = stem.as_ref().with_extension(extension);
        if let Ok(t) = from_file(&path) {
            return Ok(t);
        }
    }

    Err(Error::NoSuccessfulParse)
}

#[allow(unused_mut)]
pub fn to_string<T>(value: &T, format: Format) -> Result<String, Error>
where
    T: Serialize,
{
    #[allow(unreachable_patterns)]
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => Ok(serde_yaml::to_string(value)?),
        #[cfg(feature = "json")]
        Format::Json => Ok(serde_json::to_string(value)?),
        #[cfg(feature = "toml")]
        Format::Toml => Ok(toml::to_string(value)?),
        #[cfg(feature = "ron")]
        Format::Ron => Ok(ron::ser::to_string(value)?),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

pub fn to_vec<T>(value: &T, format: Format) -> Result<Vec<u8>, Error>
where
    T: Serialize,
{
    #[allow(unreachable_patterns)]
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => Ok(serde_yaml::to_vec(value)?),
        #[cfg(feature = "json")]
        Format::Json => Ok(serde_json::to_vec(value)?),
        #[cfg(feature = "toml")]
        Format::Toml => Ok(toml::to_vec(value)?),
        #[cfg(feature = "ron")]
        Format::Toml => Ok(ron::ser::to_string(value)?.into_bytes()),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

#[allow(unused_mut)]
pub fn to_writer<W, T>(mut writer: W, value: &T, format: Format) -> Result<(), Error>
where
    W: Write,
    T: Serialize,
{
    #[allow(unreachable_patterns)]
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => Ok(serde_yaml::to_writer(writer, value)?),
        #[cfg(feature = "json")]
        Format::Json => Ok(serde_json::to_writer(writer, value)?),
        #[cfg(feature = "toml")]
        Format::Toml => {
            let s = toml::to_vec(value)?;
            writer.write(&s)?;
            Ok(())
        }
        #[cfg(feature = "ron")]
        Format::Ron => {
            let s = ron::ser::to_string(value)?;
            write!(&mut writer, "{}", s)?;
            Ok(())
        }

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
