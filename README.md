# Serde Any

Dynamic serialization and deserialization with the format chosen at runtime.
This crate allows you to serialize and deserialize data using [serde](https://serde.rs/) without having to choose the data format up-front.
The format can be either chosen at runtime, inferred from the file name, or guessed by attemting each supported format until one succeeds.

## Supported formats

By default, JSON, YAML, TOML, and RON formats are supported

```
[dependencies]
serde_any = "0.3"
```

The list of supported formats can be controlled via feature flags

```
[dependencies]
serde_any = { version = "0.3", default-features = false, features = ["yaml", "toml"] }
```

## Deserialization

Structs that implement `serde::de::Deserialize` can be deserialized from a given file

```
let m: MyStruct = serde_any::from_file("my_data.json")?;
```

If support for multiple formats is desired, data can be deserialized from a fixed file stem.
The following will look for `my_data.json`, `my_data.yaml`, `my_data.yml`, `my_data.toml`, `my_data.ron`.

```
let m: MyStruct = serde_any::from_file_stem("my_data")?;
```

Deserialization can also be done from a string, a byte array, or a reader, with any format

```
let m1: MyStruct = serde_any::from_str_any(r#"{"name": "value"}"#)?;
let m2: MyStruct = serde_any::from_slice_any(b"name: value")?;
```

The `serde_any` crate also provides similar deserialization function with a fixed format

```
let m1: MyStruct = serde_any::from_str(r#"{"name": "value"}"#, serde_any::Format::Json)?;
let m2: MyStruct = serde_any::from_slice(b"name: value", serde_any::Format::Yaml)?;
let m3: MyStruct = serde_any::from_reader(File::open("some_data_file.toml")?, serde_any::Format::Toml)?;
```

## Serialization

Structs that implement `serde::ser::Serialize` can be serialized into a file

```
    let m = MyStruct { ... };
    serde_any::to_file("my_data.json")?;
```

or to a string, a byte array, or a writer, with a fixed format


```
let m = MyStruct { ... };
let json_string = serde_any::to_string(&m, Format::Json)?;
let yaml_byte_vec = serde_any::to_vec(&m, Format::Yaml)?;

let writer = File::create("some_data_file.toml")?;
serde_any::to_writer(writer, Format::Toml)?;
```
