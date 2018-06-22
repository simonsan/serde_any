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

pub mod error;
pub use error::Error;

pub mod format;
pub use format::*;

pub mod de;
pub use de::*;

pub mod ser;
pub use ser::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::fs::{File, remove_file};
    use std::path::Path;
    use std::io::Write;

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
            friends: vec![
                "hobbits".to_string(),
                "dwarves".to_string(),
                "elves".to_string(),
                "men".to_string(),
            ],
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
            friends: vec![
                "hobbits".to_string(),
                "dwarves".to_string(),
                "elves".to_string(),
                "men".to_string(),
            ],
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

    #[test]
    fn to_cursor_and_back_again() {
        let gandalf = Wizard {
            name: "Gandalf".to_string(),
            color: "White".to_string(),
            is_late: false,
            age: 9001,
            friends: vec![
                "hobbits".to_string(),
                "dwarves".to_string(),
                "elves".to_string(),
                "men".to_string(),
            ],
        };

        let formats = vec![Format::Json, Format::Toml, Format::Yaml, Format::Ron];
        for format in formats {
            assert!(format.is_supported());

            let mut v: Vec<u8> = Vec::new();
            to_writer(Cursor::new(&mut v), &gandalf, format).unwrap();

            let gandalf_the_deserialized_from_reader: Wizard = from_reader(Cursor::new(&mut v), format).unwrap();
            assert_eq!(gandalf_the_deserialized_from_reader, gandalf);

            let gandalf_the_deserialized_from_slice: Wizard = from_slice(&v, format).unwrap();
            assert_eq!(gandalf_the_deserialized_from_slice, gandalf);

            let gandalf_the_deserialized_from_slice_any: Wizard = from_slice_any(&v).unwrap();
            assert_eq!(gandalf_the_deserialized_from_slice_any, gandalf);
        }
    }

    #[test]
    fn to_file_and_back_again() {
        let gandalf = Wizard {
            name: "Gandalf".to_string(),
            color: "White".to_string(),
            is_late: false,
            age: 9001,
            friends: vec![
                "hobbits".to_string(),
                "dwarves".to_string(),
                "elves".to_string(),
                "men".to_string(),
            ],
        };

        let extensions = vec!["json", "toml", "yaml", "ron"];
        let stem = Path::new("gandalf_1");
        for ext in extensions {
            let file_name = stem.with_extension(ext);
            to_file(&file_name, &gandalf).unwrap();
            let gandalf_the_deserialized: Wizard = from_file(&file_name).unwrap();
            remove_file(&file_name).unwrap();
            assert_eq!(gandalf_the_deserialized, gandalf);
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

    macro_rules! assert_pattern {
        ($value:expr, $pattern:pat, $message:expr) => {
            match $value {
                $pattern => {},
                r => panic!("Expected {}, got result {:?}", $message, r),
            }
        }
    }

    #[test]
    fn test_assert_pattern_success() {
        let e: Result<(), Error> = Err(Error::NoSuccessfulParse);
        assert_pattern!(e, Err(Error::NoSuccessfulParse), "Error::NoSuccessfulParse");
    }

    #[test]
    #[should_panic]
    fn test_assert_pattern_panic() {
        let e: Result<(), Error> = Ok(());
        assert_pattern!(e, Err(Error::NoSuccessfulParse), "Error::NoSuccessfulParse");
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
    fn valid_but_unknown_extension() {
        let gandalf = Wizard {
            name: "Gandalf".to_string(),
            color: "White".to_string(),
            is_late: false,
            age: 9001,
            friends: vec![
                "hobbits".to_string(),
                "dwarves".to_string(),
                "elves".to_string(),
                "men".to_string(),
            ],
        };

        let json = to_vec(&gandalf, Format::Json).unwrap();
        let file_name = "gandalf_2.dat";
        {
            let mut file = File::create(file_name).unwrap();
            file.write(&json).unwrap();
        }

        {
            let gandalf_the_deserialized: Wizard = from_file(file_name).unwrap();
            assert_eq!(gandalf_the_deserialized, gandalf);
        }

        remove_file(file_name).unwrap();
    }

    #[test]
    fn valid_file_stem() {
        let gandalf = Wizard {
            name: "Gandalf".to_string(),
            color: "White".to_string(),
            is_late: false,
            age: 9001,
            friends: vec![
                "hobbits".to_string(),
                "dwarves".to_string(),
                "elves".to_string(),
                "men".to_string(),
            ],
        };

        for ext in supported_extensions() {
            let file_name = Path::new("gandalf_3").with_extension(ext);
            to_file(&file_name, &gandalf).unwrap();

            let gandalf_the_deserialized: Wizard = from_file_stem("gandalf_3").unwrap();
            assert_eq!(gandalf_the_deserialized, gandalf);

            remove_file(&file_name).unwrap();
        }
    }

    #[test]
    fn unknown_extension_write() {
        let gandalf = Wizard {
            name: "Gandalf".to_string(),
            color: "White".to_string(),
            is_late: false,
            age: 9001,
            friends: vec![
                "hobbits".to_string(),
                "dwarves".to_string(),
                "elves".to_string(),
                "men".to_string(),
            ],
        };

        let file_name = "gandalf_4.dat";
        assert_pattern!(
            to_file(file_name, &gandalf),
            Err(Error::UnsupportedFileExtension(_)),
            "Error::UnsupportedFileExtension"
        );
        remove_file(file_name).ok();
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
