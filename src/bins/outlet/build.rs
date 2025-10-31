use glob::glob;
use std::{env, error::Error, path::PathBuf};

const PROTO_TARGET: &str = "harvester";

fn main() -> Result<(), Box<dyn Error>> {
    let proto_files: Vec<_> = glob(&format!("../../proto/{}/*.proto", PROTO_TARGET))?
        .filter_map(Result::ok)
        .collect();

    if proto_files.is_empty() {
        return Err(format!("No .proto files found in ../../proto/{}/", PROTO_TARGET).into());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Fix: Use the correct include path: "../../proto"
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join(format!("{}_descriptor.bin", PROTO_TARGET)))
        // .build_server(true)
        .build_client(true)
        .compile_protos(
            &proto_files,
            &[format!("../../proto/{}", PROTO_TARGET).into()],
        )?; // ← Fixed here

    Ok(())
}
