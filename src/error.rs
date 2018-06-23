use std;

use backend::*;
use format::Format;

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
