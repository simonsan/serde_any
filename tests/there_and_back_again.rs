extern crate serde_any;

#[macro_use]
extern crate serde;

use serde_any::*;

use std::io::Cursor;
use std::fs::{remove_file, File};
use std::path::Path;
use std::io::Write;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Hobbit {
    pub name: String,
    pub age: u32,
    pub has_ring: bool,
}

pub fn young_bilbo() -> Hobbit {
    Hobbit {
        name: "Bilbo Baggins".to_string(),
        age: 50,
        has_ring: false,
    }
}

pub fn old_bilbo() -> Hobbit {
    Hobbit {
        name: "Bilbo Baggins".to_string(),
        age: 111,
        has_ring: true,
    }
}

pub fn all_formats() -> Vec<Format> {
    vec![
        Format::Json,
        Format::Toml,
        Format::Yaml,
        Format::Ron,
        Format::Xml,
        Format::Url,
    ]
}

#[test]
fn to_vec_and_back_and_to_vec_again() {
    let bilbo = young_bilbo();

    for format in all_formats() {
        assert!(format.is_supported());

        let bilbo_the_serialized = to_vec(&bilbo, format).unwrap();
        let bilbo_the_deserialized: Hobbit = from_slice(&bilbo_the_serialized, format).unwrap();
        assert_eq!(bilbo_the_deserialized, bilbo);
        let bilbo_the_serialized_again = to_vec(&bilbo_the_deserialized, format).unwrap();
        assert_eq!(bilbo_the_serialized_again, bilbo_the_serialized);
    }
}

#[test]
fn to_string_and_back_and_to_string_again() {
    let bilbo = old_bilbo();

    for format in all_formats() {
        assert!(format.is_supported());

        let bilbo_the_serialized = to_string(&bilbo, format).unwrap();
        let bilbo_the_deserialized: Hobbit = from_str(&bilbo_the_serialized, format).unwrap();
        assert_eq!(bilbo_the_deserialized, bilbo);
        let bilbo_the_serialized_again = to_string(&bilbo_the_deserialized, format).unwrap();
        assert_eq!(bilbo_the_serialized_again, bilbo_the_serialized);
    }
}

#[test]
fn to_cursor_and_back_again() {
    let bilbo = old_bilbo();

    for format in all_formats() {
        assert!(format.is_supported());

        let mut v: Vec<u8> = Vec::new();
        to_writer(Cursor::new(&mut v), &bilbo, format).unwrap();

        let bilbo_the_deserialized_from_reader: Hobbit = from_reader(Cursor::new(&mut v), format).unwrap();
        assert_eq!(bilbo_the_deserialized_from_reader, bilbo);

        let bilbo_the_deserialized_from_slice: Hobbit = from_slice(&v, format).unwrap();
        assert_eq!(bilbo_the_deserialized_from_slice, bilbo);

        let bilbo_the_deserialized_from_slice_any: Hobbit = from_slice_any(&v).unwrap();
        assert_eq!(bilbo_the_deserialized_from_slice_any, bilbo);
    }
}

#[test]
fn to_file_and_back_again() {
    let bilbo = old_bilbo();

    let extensions = vec!["json", "toml", "yaml", "ron"];
    let stem = Path::new("bilbo_1");
    for ext in extensions {
        let file_name = stem.with_extension(ext);
        to_file(&file_name, &bilbo).unwrap();
        let bilbo_the_deserialized: Hobbit = from_file(&file_name).unwrap();
        remove_file(&file_name).unwrap();
        assert_eq!(bilbo_the_deserialized, bilbo);
    }
}

#[test]
fn to_vec_and_back_and_to_vec_again_pretty() {
    let bilbo = young_bilbo();

    for format in all_formats() {
        assert!(format.is_supported());

        let bilbo_the_serialized = to_vec_pretty(&bilbo, format).unwrap();
        let bilbo_the_deserialized: Hobbit = from_slice(&bilbo_the_serialized, format).unwrap();
        assert_eq!(bilbo_the_deserialized, bilbo);
        let bilbo_the_serialized_again = to_vec_pretty(&bilbo_the_deserialized, format).unwrap();
        assert_eq!(bilbo_the_serialized_again, bilbo_the_serialized);
    }
}

#[test]
fn to_string_and_back_and_to_string_again_pretty() {
    let bilbo = old_bilbo();

    for format in all_formats() {
        assert!(format.is_supported());

        let bilbo_the_serialized = to_string_pretty(&bilbo, format).unwrap();
        let bilbo_the_deserialized: Hobbit = from_str(&bilbo_the_serialized, format).unwrap();
        assert_eq!(bilbo_the_deserialized, bilbo);
        let bilbo_the_serialized_again = to_string_pretty(&bilbo_the_deserialized, format).unwrap();
        assert_eq!(bilbo_the_serialized_again, bilbo_the_serialized);
    }
}

#[test]
fn to_cursor_and_back_again_pretty() {
    let bilbo = old_bilbo();

    for format in all_formats() {
        assert!(format.is_supported());

        let mut v: Vec<u8> = Vec::new();
        to_writer_pretty(Cursor::new(&mut v), &bilbo, format).unwrap();

        let bilbo_the_deserialized_from_reader: Hobbit = from_reader(Cursor::new(&mut v), format).unwrap();
        assert_eq!(bilbo_the_deserialized_from_reader, bilbo);

        let bilbo_the_deserialized_from_slice: Hobbit = from_slice(&v, format).unwrap();
        assert_eq!(bilbo_the_deserialized_from_slice, bilbo);

        let bilbo_the_deserialized_from_slice_any: Hobbit = from_slice_any(&v).unwrap();
        assert_eq!(bilbo_the_deserialized_from_slice_any, bilbo);
    }
}

#[test]
fn to_file_and_back_again_pretty() {
    let bilbo = old_bilbo();

    let extensions = vec!["json", "toml", "yaml", "ron"];
    let stem = Path::new("bilbo_5");
    for ext in extensions {
        let file_name = stem.with_extension(ext);
        to_file_pretty(&file_name, &bilbo).unwrap();
        let bilbo_the_deserialized: Hobbit = from_file(&file_name).unwrap();
        remove_file(&file_name).unwrap();
        assert_eq!(bilbo_the_deserialized, bilbo);
    }
}

#[test]
fn valid_but_unknown_extension() {
    let bilbo = old_bilbo();

    let json = to_vec(&bilbo, Format::Json).unwrap();
    let file_name = "bilbo_2.dat";
    {
        let mut file = File::create(file_name).unwrap();
        file.write(&json).unwrap();
    }

    {
        let bilbo_the_deserialized: Hobbit = from_file(file_name).unwrap();
        assert_eq!(bilbo_the_deserialized, bilbo);
    }

    remove_file(file_name).unwrap();
}

#[test]
fn valid_file_stem() {
    let bilbo = old_bilbo();

    for ext in supported_extensions() {
        let file_name = Path::new("bilbo_3").with_extension(ext);
        to_file(&file_name, &bilbo).unwrap();

        let bilbo_the_deserialized: Hobbit = from_file_stem("bilbo_3").unwrap();
        assert_eq!(bilbo_the_deserialized, bilbo);

        remove_file(&file_name).unwrap();
    }
}
