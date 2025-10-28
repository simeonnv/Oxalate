use glob::glob;
use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let proto_files: Vec<_> = glob("../../proto/harvester/*.proto")?
        .filter_map(Result::ok)
        .collect();

    if proto_files.is_empty() {
        return Err("No .proto files found in ../../proto/harvester/".into());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Fix: Use the correct include path: "../../proto"
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("harvester_descriptor.bin"))
        .build_server(true)
        .build_client(true)
        .compile_protos(&proto_files, &["../../proto/harvester".into()])?; // ← Fixed here

    Ok(())
}
