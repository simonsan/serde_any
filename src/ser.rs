use serde::ser::Serialize;

use std::fs::File;
use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;

use backend::*;
use format::{guess_format, Format};
use error::Error;

/// Serialize to a `String`
///
/// # Errors
///
/// If serialization fails, the format-specific [`Error`] variant is returned,
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
///
/// [`Error`]: ../error/enum.Error.html
///
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
        #[cfg(feature = "xml")]
        Format::Xml => Ok(xml::to_string(value)?),
        #[cfg(feature = "url")]
        Format::Url => Ok(url::to_string(value)?),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

/// Serialize to a pretty-printed `String`
///
/// Not all serialization formats support pretty printing.
/// In such cases, the output from this function will be identical to the output
/// of [`to_string`].
///
/// # Errors
///
/// If serialization fails, the format-specific [`Error`] variant is returned,
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
///     let data = serde_any::to_string_pretty(&bran, Format::Toml)?;
///     println!("{}", data);
///     assert_eq!(&data[..], "name = \'Brandon Stark\'\nknowledge = 100\n");
///     Ok(())
/// }
/// ```
///
/// [`Error`]: ../error/enum.Error.html
/// [`to_string`]: fn.to_string.html
///
#[allow(unused_mut)]
pub fn to_string_pretty<T>(value: &T, format: Format) -> Result<String, Error>
where
    T: Serialize,
{
    #[allow(unreachable_patterns)]
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => Ok(serde_yaml::to_string(value)?),
        #[cfg(feature = "json")]
        Format::Json => Ok(serde_json::to_string_pretty(value)?),
        #[cfg(feature = "toml")]
        Format::Toml => Ok(toml::to_string_pretty(value)?),
        #[cfg(feature = "ron")]
        Format::Ron => Ok(ron::ser::to_string_pretty(
            value,
            ron::ser::PrettyConfig::default(),
        )?),
        #[cfg(feature = "xml")]
        Format::Xml => Ok(xml::to_string(value)?),
        #[cfg(feature = "url")]
        Format::Url => Ok(url::to_string(value)?),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

/// Serialize to a byte vector
///
/// # Errors
///
/// If serialization fails, the format-specific [`Error`] variant is returned,
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
///
/// [`Error`]: ../error/enum.Error.html
///
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
        #[cfg(feature = "xml")]
        Format::Xml => Ok(xml::to_string(value)?.into_bytes()),
        #[cfg(feature = "url")]
        Format::Url => Ok(url::to_string(value)?.into_bytes()),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

/// Serialize to a pretty-printed byte vector
///
/// Not all serialization formats support pretty printing.
/// In such cases, the output from this function will be identical to the output
/// of [`to_vec`].
///
/// # Errors
///
/// If serialization fails, the format-specific [`Error`] variant is returned,
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
///     let data = serde_any::to_vec_pretty(&bran, Format::Toml)?;
///     assert_eq!(
///         data,
///         b"name = \'Brandon Stark\'\nknowledge = 100\n".to_vec()
///     );
///     Ok(())
/// }
/// ```
///
/// [`Error`]: ../error/enum.Error.html
/// [`to_vec`]: fn.to_vec.html
///
pub fn to_vec_pretty<T>(value: &T, format: Format) -> Result<Vec<u8>, Error>
where
    T: Serialize,
{
    #[allow(unreachable_patterns)]
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => Ok(serde_yaml::to_vec(value)?),
        #[cfg(feature = "json")]
        Format::Json => Ok(serde_json::to_vec_pretty(value)?),
        #[cfg(feature = "toml")]
        Format::Toml => Ok(toml::ser::to_string_pretty(value)?.into_bytes()),
        #[cfg(feature = "ron")]
        Format::Ron => Ok(ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())?.into_bytes()),
        #[cfg(feature = "xml")]
        Format::Xml => Ok(xml::to_string(value)?.into_bytes()),
        #[cfg(feature = "url")]
        Format::Url => Ok(url::to_string(value)?.into_bytes()),

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

/// Serialize to a writer
///
/// # Errors
///
/// If serialization fails, the format-specific [`Error`] variant is returned,
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
///     std::fs::remove_file("weirwood.ron").unwrap();
///     let data = serde_any::to_writer(file, &bran, Format::Ron)?;
///     Ok(())
/// }
/// ```
///
/// [`Error`]: ../error/enum.Error.html
///
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
        #[cfg(feature = "xml")]
        Format::Xml => Ok(xml::to_writer(writer, value)?),
        #[cfg(feature = "url")]
        Format::Url => {
            let s = url::to_string(value)?;
            write!(&mut writer, "{}", s)?;
            Ok(())
        }

        _ => Err(Error::UnsupportedFormat(format)),
    }
}

/// Serialize to a writer with pretty printing
///
/// Not all serialization formats support pretty printing.
/// In such cases, the output from this function will be identical to the output
/// of [`to_writer`].
///
/// # Errors
///
/// If serialization fails, the format-specific [`Error`] variant is returned,
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
///     std::fs::remove_file("weirwood.ron").unwrap();
///     let data = serde_any::to_writer_pretty(file, &bran, Format::Ron)?;
///     Ok(())
/// }
/// ```
///
/// [`Error`]: ../error/enum.Error.html
/// [`to_writer`]: fn.to_writer.html
///
#[allow(unused_mut)]
pub fn to_writer_pretty<W, T>(mut writer: W, value: &T, format: Format) -> Result<(), Error>
where
    W: Write,
    T: Serialize,
{
    #[allow(unreachable_patterns)]
    match format {
        #[cfg(feature = "yaml")]
        Format::Yaml => Ok(serde_yaml::to_writer(writer, value)?),
        #[cfg(feature = "json")]
        Format::Json => Ok(serde_json::to_writer_pretty(writer, value)?),
        #[cfg(feature = "toml")]
        Format::Toml => {
            let s = toml::to_string_pretty(value)?;
            write!(&mut writer, "{}", s)?;
            Ok(())
        }
        #[cfg(feature = "ron")]
        Format::Ron => {
            let s = ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())?;
            write!(&mut writer, "{}", s)?;
            Ok(())
        }
        #[cfg(feature = "xml")]
        Format::Xml => Ok(xml::to_writer(writer, value)?),
        #[cfg(feature = "url")]
        Format::Url => {
            let s = url::to_string(value)?;
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
/// [`Error::UnsupportedFileExtension`] is returned.
///
/// If opening the file for writing fails, [`Error::Io`] is returned.
///
/// If serialization fails, the format-specific [`Error`] variant is returned,
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
///     # std::fs::remove_file("bran.yaml").unwrap();
///     Ok(())
/// }
/// ```
///
/// [`Error`]: ../error/enum.Error.html
/// [`Error::UnsupportedFileExtension`]: ../error/enum.Error.html#variant.UnsupportedFileExtension
/// [`Error::Io`]: ../error/enum.Error.html#variant.Io
///
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

/// Serialize to a file with pretty printing
///
/// Not all serialization formats support pretty printing.
/// In such cases, the output from this function will be identical to the output
/// of [`to_file`].
///
/// # Errors
///
/// If the serialization format cannot be inferred from the file name,
/// [`Error::UnsupportedFileExtension`] is returned.
///
/// If opening the file for writing fails, [`Error::Io`] is returned.
///
/// If serialization fails, the format-specific [`Error`] variant is returned,
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
///     serde_any::to_file_pretty("bran.yaml", &bran)?;
///     # std::fs::remove_file("bran.yaml").unwrap();
///     Ok(())
/// }
/// ```
///
/// [`Error`]: ../error/enum.Error.html
/// [`Error::UnsupportedFileExtension`]: ../error/enum.Error.html#variant.UnsupportedFileExtension
/// [`Error::Io`]: ../error/enum.Error.html#variant.Io
/// [`to_file`]: fn.to_file.html
///
pub fn to_file_pretty<T, P>(path: P, value: &T) -> Result<(), Error>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let format = guess_format(&path);

    match format {
        Some(format) => to_writer_pretty(File::create(path)?, value, format),
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
    use std::fs::remove_file;

    #[derive(Serialize, PartialEq, Debug)]
    struct Foo {
        size: usize,
        bar: Vec<f32>,
    }

    #[test]
    fn unknown_extension_write() {
        let foo = Foo {
            size: 10,
            bar: vec![1.0, 2.0, 4.0, 8.0],
        };

        let file_name = "ser_foo.dat";
        assert_matches!(
            to_file(file_name, &foo),
            Err(Error::UnsupportedFileExtension(_))
        );
        assert_matches!(
            to_file_pretty(file_name, &foo),
            Err(Error::UnsupportedFileExtension(_))
        );
        remove_file(file_name).ok();
    }
}
