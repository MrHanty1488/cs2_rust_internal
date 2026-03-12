use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");
    let sdk_server_dll = Path::new(&manifest_dir).join("src").join("sdk").join("server_dll.rs");
    let out_dir_value = env::var("OUT_DIR").expect("OUT_DIR is set");
    let dst_dir = Path::new(&out_dir_value);
    let dst_file = dst_dir.join("server_dll.rs");

    let data = fs::read_to_string(&sdk_server_dll)?;
    let mut skipped_attr = false;

    let mut out = String::new();

    for line in data.lines() {
        if !skipped_attr {
            let trimmed = line.trim_start();
            if trimmed.starts_with("#![allow(") {
                skipped_attr = true;
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }

    fs::write(&dst_file, out)?;
    println!("cargo:rerun-if-changed={}", sdk_server_dll.display());
    Ok(())
}
