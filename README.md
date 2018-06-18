# Serde Any

Dynamic serialization and deserialization with the format chosen at runtime.
This crate allows you to serialize and deserialize data using [serde](https://serde.rs/) without having to choose the data format up-front.
The format can be either chosen at runtime, inferred from the file name, or guessed by attemting each supported format until one succeeds.

## Usage

By default, JSON, YAML, TOML, and RON formats are supported

```
[dependencies]
serde_any = "0.1"
```

The list of supported formats can be controlled via feature flags

```
[dependencies]
serde_any = { version = "0.1", default-features = false, features = ["yaml", "toml"] }
```
