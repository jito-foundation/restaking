#[cfg(feature = "shrink")]
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

#[cfg(feature = "shrink")]
use flate2::{write::ZlibEncoder, Compression};

#[cfg(feature = "shrink")]
pub fn compress_idl(idl_path: &PathBuf, dest_path: &PathBuf) {
    let mut idl_json = File::open(idl_path).unwrap();
    let mut json_contents = String::new();
    idl_json
        .read_to_string(&mut json_contents)
        .expect("Failed to read JSON file");

    // Compress the JSON contents using zlib
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(json_contents.as_bytes())
        .expect("Failed to compress JSON data");
    let compressed_data = encoder.finish().expect("Failed to finish compression");

    // Get the output directory for the build script
    // Write the compressed data to a file in the output directory
    let mut output_file = File::create(dest_path).expect("Failed to create output file");
    output_file
        .write_all(&compressed_data)
        .expect("Failed to write compressed data to file");
}
