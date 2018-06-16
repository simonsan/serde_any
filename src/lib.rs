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

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "JSON error: {}", _0)] Json(#[fail(cause)] serde_json::Error),

    #[fail(display = "YAML error: {}", _0)] Yaml(#[fail(cause)] serde_yaml::Error),

    #[fail(display = "TOML deserialize error: {}", _0)]
    TomlDeserialize(#[fail(cause)] toml::de::Error),

    #[fail(display = "TOML serialize error: {}", _0)]
    TomlSerialize(#[fail(cause)] toml::ser::Error),

    #[fail(display = "RON deserialize error: {}", _0)]
    RonDeserialize(#[fail(cause)] ron::de::Error),

    #[fail(display = "RON serialize error: {}", _0)] RonSerialize(#[fail(cause)] ron::ser::Error),

    #[fail(display = "IO error: {}", _0)] Io(#[fail(cause)] std::io::Error),

    #[fail(display = "Format {} not supported", _0)] UnsupportedFormat(Format),

    #[fail(display = "File extension {} not supported", _0)] UnsupportedFileExtension(String),

    #[fail(display = "No format was able to parse the source")] NoSuccessfulParse,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Toml,
    Json,
    Yaml,
    Ron,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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

pub fn guess_format<P>(path: P) -> Option<Format>
where
    P: AsRef<Path>,
{
    path.as_ref()
        .extension()
        .and_then(OsStr::to_str)
        .and_then(guess_format_from_extension)
}

pub fn guess_format_from_extension(ext: &str) -> Option<Format> {
    match ext {
        "yml" | "yaml" => Some(Format::Yaml),
        "json" => Some(Format::Json),
        "toml" => Some(Format::Toml),
        "ron" => Some(Format::Ron),
        _ => None,
    }
}

pub fn from_reader<T, R>(mut reader: R, format: Format) -> Result<T, Error>
where
    T: DeserializeOwned,
    R: Read,
{
    #[allow(unreachable_patterns)]
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
