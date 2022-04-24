use std::{path::{Path}, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    
    // Paths to asset files and external source files
    let extern_path = Path::new("extern/");
    let asset_path = Path::new("assets/");

    // Build par_shapes C library (https://github.com/prideout/par/blob/master/par_shapes.h)
    let par_shape_src_path = extern_path.join("par_shape.cpp");
    cc::Build::new().file(par_shape_src_path).compile("par_shape");
    
    // Move hlsl files to build out dir
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
