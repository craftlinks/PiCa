use std::{path::{Path}, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let asset_path = Path::new("assets/");

    for _entry in std::fs::read_dir(asset_path)? {
        let entry = _entry?;
        let file_name = entry.file_name().into_string().unwrap();
        if file_name.ends_with(".hlsl") {
            let shaders_hlsl_path = asset_path.join(file_name.as_str());
            let out_file = format!("/../../../{}", file_name.as_str());
            let _out_path = std::env::var("OUT_DIR").unwrap() + &out_file;
            let out_path = Path::new(_out_path.as_str());
            std::fs::copy(&shaders_hlsl_path, out_path).expect("Copy");
            println!("!cargo:rerun-if-changed={:?}" , &shaders_hlsl_path);
        }
    }
    Ok(())
}
