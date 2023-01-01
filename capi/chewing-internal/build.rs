use std::env;
use std::error::Error;
use std::path::PathBuf;

const C_HEADER_OUTPUT: &str = "chewing_internal.h";

fn main() -> Result<(), Box<dyn Error>> {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").or(Err("CARGO_MANIFEST_DIR not specified"))?;
    let build_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| ".".into()));
    let outfile_path = build_dir.join(C_HEADER_OUTPUT);

    // Useful for build diagnostics
    eprintln!("cbindgen outputting {:?}", &outfile_path);
    cbindgen::generate(crate_dir)
        .expect("Unable to generate C headers for Rust code")
        .write_to_file(&outfile_path);

    Ok(())
}
