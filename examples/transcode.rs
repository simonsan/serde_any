extern crate serde;
extern crate serde_any;
extern crate serde_value;

#[macro_use]
extern crate structopt;

extern crate failure;

use structopt::StructOpt;

use std::path::PathBuf;

#[derive(StructOpt, Clone, Debug)]
struct Opt {
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input: PathBuf,
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,
}

//
// A program that will "transcode" a House object between different
// serialization formats.
//
// The user can specify `input` and `output` paths with any supported file
// extension, and the program will use it to choose the deserialization and
// serialization format.
//
// For example, `transcode -i house.toml -o house.json` will convert it from
// TOML to JSON.
//

fn main() -> Result<(), failure::Error> {
    let opt = Opt::from_args();

    println!("Transcoding from '{}' to '{}'", opt.input.display(), opt.output.display());

    // Reads a Value from the given input file.
    // The format is chosen according to the file extension if possible,
    // otherwise all supported formats are attempted
    let value: serde_value::Value = serde_any::from_file(&opt.input)?;

    // Writes the Value to the given output file,
    // with the format chosen according to the file extension.
    serde_any::to_file(&opt.output, &value)?;

    Ok(())
}
