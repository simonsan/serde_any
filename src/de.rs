use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::de::{Deserialize, DeserializeOwned};

#[cfg(feature = "json")]
use serde_json;

#[cfg(feature = "yaml")]
use serde_yaml;

#[cfg(feature = "toml")]
use toml;

#[cfg(feature = "ron")]
use ron;

use format::{Format, supported_formats, supported_extensions, guess_format};
use error::Error;

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
