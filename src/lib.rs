#![warn(missing_docs)]

//! # Serde Any
//!
//! Dynamic serialization and deserialization with the format chosen at runtime
//!
//! ## Deserialization with a known format
//!
//! If the deserialization format is known in advance, `serde_any` mirrors the API of `serde_json` and `serde_yaml`.
//! Namely, functions [`from_reader`](fn.from_reader.html), [`from_slice`](fn.from_slice.html) and
//! [`from_str`](fn.from_str.html) work in the same way as those in format-specific crates, except that they take an
//! additional [`Format`](enum.Format.html) paramater specifying the deserialization format. The
//! [`from_file`](fn.from_file.html) function is provided as a convenience wrapper around
//! [`from_reader`](fn.from_reader.html) for the common case of reading from a file.
//!
//! ## Deserialization by guessing
//!
//! This crate also supports deserialization where the data format is not known in advance.
//! There are three different ways of inferring the data format:
//! * with [`from_file`](fn.from_file.html), the format is deduced from the file extension.
//!   This is useful if a user can load a data file of any format.
//! * with [`from_file_stem`](fn.from_file_stem.html), each filename with the given stem and a supported extension
//!   is checked. If any such file exists, its data is deserialized and returned.
//!   This is useful for configuration files with a known set of filenames.
//! * with [`from_slice_any`](fn.from_slice_any.html) and [`from_str_any`](fn.from_str_any.html), deserialization
//!   using each supported format is tried until one succeeds.
//!   This is useful when you receive data from an unknown source and don't know what format it is in.
//!
//! Note there is no corresponding `from_reader_any` function, as attempting to deserialize from a reader would
//! consume its data. In order to deserialize from a `std::io::Read`, read the data into a `Vec<u8>` or `String` and
//! call [`from_slice_any`](fn.from_slice_any) or [`from_str_any`](fn.from_str_any.html).
//!
//! ## Serialization
//!
//! For serialization, the data format must always be provided.
//! Consistent with the format-specific crates, data may be serialized to a `String` with
//! [`to_string`](fn.to_string.html), to a `Vec<u8>` with [`to_vec`](fn.to_vec.html), or to a `std::io::Write` with
//! [`to_writer`](fn.to_writer.html).
//!
//! Alternatively, when writing to a file, the format can be inferred from the file name by the
//! [`to_file`](fn.to_file.html) function. Similarly to [`from_file`](fn.from_file.html), this is most useful when
//! saving to a user-selected file.
//!
//! There is no support for pretty-printing yet.
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

#[cfg(test)]
#[macro_use]
extern crate serde_derive;

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
    /// when building `serde_any`.
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
/// \"name\": \"Jon Snow\",
/// \"knowledge\": 0
/// }";
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
/// \"name\": \"Jon Snow\",
/// \"knowledge\": 0
/// }";
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
/// \"name\": \"Jon Snow\",
/// \"knowledge\": 0
/// }";
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
/// \"name\": \"Jon Snow\",
/// \"knowledge\": 0
/// }";
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

/// Deserialize from a file
///
/// The format is detected using `guess_format`.
/// If that fails, such as if the file extension is not recognized,
/// the whole file is read into a buffer,
/// and deserialization is attempted using `from_slice_any`.
///
/// # Errors
///
/// If the file extension is recognized, but parsing fails, this function returns
/// the error from `from_reader`.
///
/// If the file extension is not recognized and the file cannot be opened,
/// it returns `Error::Io` with the underlying error as the cause.
///
/// If the file extension is not recognized, the file can opened but deserialization fails,
/// this function returns the error from `from_slice_any`.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn main() {
///     match serde_any::from_file::<User, _>("test.json") {
///         Ok(u) => println!("{:#?}", u),
///         Err(e) => println!("Error deserializing user: {}", e),
///     };
/// }
/// ```
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

/// Deserialize from any file with a given stem
///
/// This function tries to deserialize from any file with stem `stem` and any of the supported extensions.
/// The list of supported extensions can be queried with `supported_extensions`.
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
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn main() {
///     // Will attempt "user.json", "user.yaml", "user.toml" and "user.ron"
///     // If any of the features is disabled, that extension is skipped
///     match serde_any::from_file_stem::<User, _>("user") {
///         Ok(u) => println!("{:#?}", u),
///         Err(e) => println!("Error deserializing user: {}", e),
///     };
/// }
/// ```
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

/// Serialize to a `String`
///
/// # Errors
///
/// If serialization fails, the format-specific error type is returned,
/// with the underlying error as its cause.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
/// extern crate failure;
///
/// use serde_any::Format;
/// use failure::Error;
///
/// #[derive(Serialize, Debug)]
/// struct Person {
///     name: String,
///     knowledge: u32,
/// }
///
/// fn main() -> Result<(), Error> {
///     let bran = Person {
///         name: "Brandon Stark".to_string(),
///         knowledge: 100,
///     };
///     let data = serde_any::to_string(&bran, Format::Toml)?;
///     println!("{}", data);
///     assert_eq!(&data[..], "name = \"Brandon Stark\"\nknowledge = 100\n");
///     Ok(())
/// }
/// ```
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

/// Serialize to a byte vector
///
/// # Errors
///
/// If serialization fails, the format-specific error type is returned,
/// with the underlying error as its cause.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
/// extern crate failure;
///
/// use serde_any::Format;
/// use failure::Error;
///
/// #[derive(Serialize, Debug)]
/// struct Person {
///     name: String,
///     knowledge: u32,
/// }
///
/// fn main() -> Result<(), Error> {
///     let bran = Person {
///         name: "Brandon Stark".to_string(),
///         knowledge: 100,
///     };
///     let data = serde_any::to_vec(&bran, Format::Toml)?;
///     assert_eq!(
///         data,
///         b"name = \"Brandon Stark\"\nknowledge = 100\n".to_vec()
///     );
///     Ok(())
/// }
/// ```
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
        Format::Ron => Ok(ron::ser::to_string(value)?.into_bytes()),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

/// Serialize to a writer
///
/// # Errors
///
/// If serialization fails, the format-specific error type is returned,
/// with the underlying error as its cause.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
/// extern crate failure;
///
/// use serde_any::Format;
/// use failure::Error;
///
/// use std::fs::File;
///
/// #[derive(Serialize, Debug)]
/// struct Person {
///     name: String,
///     knowledge: u32,
/// }
///
/// fn main() -> Result<(), Error> {
///     let bran = Person {
///         name: "Brandon Stark".to_string(),
///         knowledge: 100,
///     };
///     let file = File::create("weirwood.ron")?;
///     let data = serde_any::to_writer(file, &bran, Format::Ron)?;
///     Ok(())
/// }
/// ```
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

/// Serialize to a file
///
/// # Errors
///
/// If the serialization format cannot be inferred from the file name,
/// `UnsupportedFileExtension` is returned.
///
/// If serialization fails, the format-specific error type is returned,
/// with the underlying error as its cause.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate serde;
/// extern crate serde_any;
/// extern crate failure;
///
/// use serde_any::Format;
/// use failure::Error;
///
/// use std::fs::File;
///
/// #[derive(Serialize, Debug)]
/// struct Person {
///     name: String,
///     knowledge: u32,
/// }
///
/// fn main() -> Result<(), Error> {
///     let bran = Person {
///         name: "Brandon Stark".to_string(),
///         knowledge: 100,
///     };
///     serde_any::to_file("bran.yaml", &bran)?;
///     Ok(())
/// }
/// ```
pub fn to_file<T, P>(path: P, value: &T) -> Result<(), Error>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let format = guess_format(&path);

    match format {
        Some(format) => to_writer(File::create(path)?, value, format),
        None => {
            let ext = path.as_ref()
                .extension()
                .and_then(OsStr::to_str)
                .map(String::from)
                .unwrap_or(String::new());
            Err(Error::UnsupportedFileExtension(ext))
        }
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

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Wizard {
        name: String,
        is_late: bool,
        color: String,
        age: u32,
        friends: Vec<String>,
    }

    #[test]
    fn to_vec_and_back_and_to_vec_again() {
        let gandalf = Wizard {
            name: "Gandalf".to_string(),
            color: "Grey".to_string(),
            is_late: false,
            age: 9000,
            friends: vec!["hobbits".to_string(), "dwarves".to_string(), "elves".to_string(), "men".to_string()],
        };

        let formats = vec![Format::Json, Format::Toml, Format::Yaml, Format::Ron];
        for format in formats {
            assert!(format.is_supported());

            let gandalf_the_serialized = to_vec(&gandalf, format).unwrap();
            let gandalf_the_deserialized: Wizard = from_slice(&gandalf_the_serialized, format).unwrap();
            assert_eq!(gandalf_the_deserialized, gandalf);
            let gandalf_the_serialized_again = to_vec(&gandalf_the_deserialized, format).unwrap();
            assert_eq!(gandalf_the_serialized_again, gandalf_the_serialized);
        }
    }

    #[test]
    fn to_string_and_back_and_to_string_again() {
        let gandalf = Wizard {
            name: "Gandalf".to_string(),
            color: "White".to_string(),
            is_late: false,
            age: 9001,
            friends: vec!["hobbits".to_string(), "dwarves".to_string(), "elves".to_string(), "men".to_string()],
        };

        let formats = vec![Format::Json, Format::Toml, Format::Yaml, Format::Ron];
        for format in formats {
            assert!(format.is_supported());

            let gandalf_the_serialized = to_string(&gandalf, format).unwrap();
            let gandalf_the_deserialized: Wizard = from_str(&gandalf_the_serialized, format).unwrap();
            assert_eq!(gandalf_the_deserialized, gandalf);
            let gandalf_the_serialized_again = to_string(&gandalf_the_deserialized, format).unwrap();
            assert_eq!(gandalf_the_serialized_again, gandalf_the_serialized);
        }
    }

    fn radagast() -> Wizard {
        Wizard {
            name: "Radagast".to_string(),
            color: "Brown".to_string(),
            is_late: true,
            age: 8000,
            friends: vec!["animals".to_string()],
        }
    }

    fn assert_deserialized_any(expected: &Wizard, s: &str) {
        let deserialized: Wizard = from_str_any(s).unwrap();
        assert_eq!(&deserialized, expected);

        let deserialized_from_bytes: Wizard = from_slice_any(s.as_bytes()).unwrap();
        assert_eq!(&deserialized_from_bytes, expected);
    }

    #[test]
    fn guess_from_json() {
        let s = r#"{"name": "Radagast", "color": "Brown", "is_late": true, "age": 8000, friends: ["animals"]}"#;
        assert_deserialized_any(&radagast(), s);
    }

    #[test]
    #[should_panic]
    fn guess_from_json_fail() {
        let s = r#"{"name" = "Radagast", "color": "Brown", "is_late": true, "age": 8000, friends: ["animals"],}"#;
        assert_deserialized_any(&radagast(), s);
    }

    #[test]
    fn guess_from_yaml_inline() {
        let s = r#"{name: Radagast, color: Brown, is_late: true, age: 8000, friends: [animals]}"#;
        assert_deserialized_any(&radagast(), s);
    }

    #[test]
    fn guess_from_yaml_long() {
        let s = "name: Radagast\ncolor: Brown\nis_late: true\nage: 8000\nfriends:\n- animals\n";
        assert_deserialized_any(&radagast(), s);
    }

    #[test]
    #[should_panic]
    fn guess_from_yaml_long_fail() {
        let s = "name: Radagast\ncolor: Brown\nis_late: true\nage: 8000\nfriends:\nanimals\n";
        assert_deserialized_any(&radagast(), s);
    }

    #[test]
    fn guess_from_toml() {
        let s = "name = \"Radagast\"\ncolor = \"Brown\"\nis_late = true\nage = 8000\nfriends = [\n  \"animals\",\n]\n";
        assert_deserialized_any(&radagast(), s);
    }

    #[test]
    fn guess_from_ron() {
        let s = "Wizard (name: \"Radagast\", color: \"Brown\", is_late: true, age: 8000, friends: [\"animals\",],)";
        assert_deserialized_any(&radagast(), s);
    }
}
