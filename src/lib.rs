#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/serde_any/0.4.0")]

//! # Serde Any
//!
//! Dynamic serialization and deserialization with the format chosen at runtime
//!
//! ## Reading from and writing to files
//!
//! ```
//! # use std::collections::HashMap;
//! # use serde_any::{Format, Error};
//! # fn main() -> Result<(), Error> {
//! let mut m = HashMap::new();
//! m.insert("a", "alpha");
//! m.insert("b", "beta");
//!
//! // Serialize to a file, the format is inferred from the file extension
//! serde_any::to_file("greek.toml", &m)?;
//!
//! // Deserialize from a file, the format is also inferred from the file extension
//! let m2: HashMap<String, String> = serde_any::from_file("greek.toml")?;
//!
//! # std::fs::remove_file("greek.toml").unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! ## Deserialization with a known format
//!
//! ```
//! # use std::collections::HashMap;
//! # use serde_any::{Format, Error};
//! # fn main() -> Result<(), Error> {
//! let d = r#"{"a": "alpha", "b": "beta"}"#;
//! let m: HashMap<String, String> = serde_any::from_str(d, Format::Json)?;
//! # assert_eq!(m.get("a"), Some(&"alpha".to_string()));
//! # assert_eq!(m.get("b"), Some(&"beta".to_string()));
//! # Ok(())
//! # }
//! ```
//!
//! If the deserialization format is known in advance, `serde_any` mirrors the API of
//! [`serde_json`](https://docs.rs/serde_json) and [`serde_yaml`](https://docs.rs/serde_yaml).
//! Namely, functions [`from_reader`], [`from_slice`] and
//! [`from_str`] work in the same way as those in format-specific crates, except that they take an
//! additional [`Format`] paramater specifying the deserialization format. The
//! [`from_file`] function is provided as a convenience wrapper around
//! [`from_reader`] for the common case of reading from a file.
//!
//! ## Deserialization by guessing
//!
//! ```
//! # use std::collections::HashMap;
//! # fn main() -> Result<(), serde_any::Error> {
//! let d = r#"{"a": "alpha", "b": "beta"}"#;
//! let m: HashMap<String, String> = serde_any::from_str_any(d)?;
//! # assert_eq!(m.get("a"), Some(&"alpha".to_string()));
//! # assert_eq!(m.get("b"), Some(&"beta".to_string()));
//! # Ok(())
//! # }
//! ```
//!
//! This crate also supports deserialization where the data format is not known in advance.
//! There are three different ways of inferring the data format:
//! * with [`from_file`], the format is deduced from the file extension.
//!   This is useful if a user can load a data file of any format.
//! * with [`from_file_stem`], each filename with the given stem and a supported extension
//!   is checked. If any such file exists, its data is deserialized and returned.
//!   This is useful for configuration files with a known set of filenames.
//! * with [`from_slice_any`] and [`from_str_any`], deserialization
//!   using each supported format is tried until one succeeds.
//!   This is useful when you receive data from an unknown source and don't know what format it is in.
//!
//! Note there is no corresponding `from_reader_any` function, as attempting to deserialize from a reader would
//! consume its data. In order to deserialize from a [`io::Read`], read the data into a [`Vec<u8>`] or [`String`] and
//! call [`from_slice_any`] or [`from_str_any`].
//!
//! ## Serialization
//!
//! ```
//! # use std::collections::HashMap;
//! # use serde_any::{Format, Error};
//! # fn main() -> Result<(), Error> {
//! let mut m = HashMap::new();
//! m.insert("a", "alpha");
//! m.insert("b", "beta");
//! let s = serde_any::to_string(&m, Format::Yaml)?;
//! # Ok(())
//! # }
//! ```
//!
//! For serialization, the data format must always be provided.
//! Consistent with the format-specific crates, data may be serialized to a [`String`] with
//! [`to_string`], to a [`Vec<u8>`] with [`to_vec`], or to a [`io::Write`] with
//! [`to_writer`].
//!
//! Alternatively, when writing to a file, the format can be inferred from the file name by the
//! [`to_file`] function. Similarly to [`from_file`], this is most useful when
//! saving to a user-selected file.
//!
//! There is no support for pretty-printing yet.
//!
//! [`Format`]: format/enum.Format.html
//! [`from_reader`]: de/fn.from_reader.html
//! [`from_slice`]: de/fn.from_slice.html
//! [`from_str`]: de/fn.from_str.html
//! [`from_file`]: de/fn.from_file.html
//! [`from_file_stem`]: de/fn.from_file_stem.html
//! [`from_slice_any`]: de/fn.from_slice_any.html
//! [`from_str_any`]: de/fn.from_str_any.html
//! [`to_string`]: ser/fn.to_string.html
//! [`to_vec`]: ser/fn.to_vec.html
//! [`to_writer`]: ser/fn.to_writer.html
//! [`to_file`]: ser/fn.to_file.html
//! [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
//! [`Vec<u8>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
//! [`io::Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
//! [`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
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

#[cfg(test)]
#[macro_use]
extern crate matches;

mod backend;

/// Contains the common error type
pub mod error;
pub use error::Error;

/// Types and functions for specifying or determining serialization formats
pub mod format;
pub use format::*;

/// Deserialize data to a Rust structure
pub mod de;
pub use de::*;

/// Serialize a Rust structure to any data format
pub mod ser;
pub use ser::*;
