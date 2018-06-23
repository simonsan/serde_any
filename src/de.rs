use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::de::{Deserialize, DeserializeOwned};

use backend::*;
use format::{guess_format, supported_extensions, supported_formats, Format};
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::*;

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

    #[test]
    fn invalid_data() {
        let s = "invalid {} data [] that cannot <> be parsed by any format !!";

        assert_pattern!(
            from_str_any::<Wizard>(&s),
            Err(Error::NoSuccessfulParse),
            "Error::NoSuccessfulParse"
        );
        assert_pattern!(
            from_slice_any::<Wizard>(s.as_bytes()),
            Err(Error::NoSuccessfulParse),
            "Error::NoSuccessfulParse"
        );
    }

    #[test]
    fn invalid_field_names() {
        let s = "name: Radagast\ncolor: Brown\nis_late: never\nage: 8000\n";

        assert_pattern!(
            from_str_any::<Wizard>(&s),
            Err(Error::NoSuccessfulParse),
            "Error::NoSuccessfulParse"
        );
        assert_pattern!(
            from_slice_any::<Wizard>(s.as_bytes()),
            Err(Error::NoSuccessfulParse),
            "Error::NoSuccessfulParse"
        );
    }

    #[test]
    fn non_existing_file() {
        assert_pattern!(
            from_file::<Wizard, _>("no_such_file.json"),
            Err(Error::Io(_)),
            "Error::Io"
        );
        assert_pattern!(
            from_file::<Wizard, _>("no_such_file.yaml"),
            Err(Error::Io(_)),
            "Error::Io"
        );
        assert_pattern!(
            from_file::<Wizard, _>("no_such_file.toml"),
            Err(Error::Io(_)),
            "Error::Io"
        );
        assert_pattern!(
            from_file::<Wizard, _>("no_such_file.ron"),
            Err(Error::Io(_)),
            "Error::Io"
        );
    }

    #[test]
    fn non_existing_file_stem() {
        assert_pattern!(
            from_file_stem::<Wizard, _>("no_such_file_stem"),
            Err(Error::NoSuccessfulParse),
            "Error::NoSuccessfulParse"
        );
    }

    #[test]
    fn empty_input_str() {
        let s = "";

        assert_pattern!(
            from_str::<Wizard>(s, Format::Json),
            Err(Error::Json(_)),
            "Error::Json"
        );
        assert_pattern!(
            from_str::<Wizard>(s, Format::Yaml),
            Err(Error::Yaml(_)),
            "Error::Yaml"
        );
        assert_pattern!(
            from_str::<Wizard>(s, Format::Toml),
            Err(Error::TomlDeserialize(_)),
            "Error::TomlDeserialize"
        );
        assert_pattern!(
            from_str::<Wizard>(s, Format::Ron),
            Err(Error::RonDeserialize(_)),
            "Error::RonDeserialize"
        );
    }

    #[test]
    fn empty_input_bytes() {
        let s = b"";

        assert_pattern!(
            from_slice::<Wizard>(s, Format::Json),
            Err(Error::Json(_)),
            "Error::Json"
        );
        assert_pattern!(
            from_slice::<Wizard>(s, Format::Yaml),
            Err(Error::Yaml(_)),
            "Error::Yaml"
        );
        assert_pattern!(
            from_slice::<Wizard>(s, Format::Toml),
            Err(Error::TomlDeserialize(_)),
            "Error::TomlDeserialize"
        );
        assert_pattern!(
            from_slice::<Wizard>(s, Format::Ron),
            Err(Error::RonDeserialize(_)),
            "Error::RonDeserialize"
        );
    }
}
