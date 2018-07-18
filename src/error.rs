use std;

use backend::*;
use format::Format;

#[cfg(feature = "xml")]
use failure::SyncFailure;

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

    /// Error deserializing with XML
    #[cfg(feature = "xml")]
    #[fail(display = "XML error: {}", _0)]
    Xml(#[fail(cause)] SyncFailure<xml::Error>),

    /// Error deserializing with URL
    #[cfg(feature = "url")]
    #[fail(display = "URL deserialize error: {}", _0)]
    UrlDeserialize(#[fail(cause)] url::de::Error),

    /// Error serializing with URL
    #[cfg(feature = "url")]
    #[fail(display = "URL serialize error: {}", _0)]
    UrlSerialize(#[fail(cause)] url::ser::Error),

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
    ///
    /// The tuple element is the list of all tried formats and the resulting errors
    #[fail(display = "No format was able to parse the source")]
    NoSuccessfulParse(Vec<(Format, Error)>),
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

#[cfg(feature = "xml")]
impl From<xml::Error> for Error {
    fn from(e: xml::Error) -> Error {
        Error::Xml(SyncFailure::new(e))
    }
}

#[cfg(feature = "url")]
impl_error_from!(url::ser::Error => Error::UrlSerialize);
#[cfg(feature = "url")]
impl_error_from!(url::de::Error => Error::UrlDeserialize);
