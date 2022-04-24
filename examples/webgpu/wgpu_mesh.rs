use pica::pica_window::{WindowAttributes, Window};
use pica::error::Error;

extern "C" {
    fn par_shapes_create_cube() -> *mut ParShapeMesh;
    fn par_shapes_create_dodecahedron() -> *mut ParShapeMesh;
}

#[repr(C)]
#[derive(Debug)]
pub struct ParShapeMesh {
    points: *mut f32,
    npoints: i32,
    triangles: *mut u16,
    ntriangles: i32,
    normals: *mut f32,
    tcoords: *mut f32,
}

pub fn main() -> Result<(), Error> {
    let cube = unsafe { &*par_shapes_create_cube() };
    println!("{:?}", cube);
    let dodeca = unsafe { &*par_shapes_create_dodecahedron() };
    println!("{:?}", dodeca);
    
    let window_attributes = WindowAttributes::new()
        .with_title("WebGPU Mesh")
        .with_position(50, 50)
        .with_size(1600, 2000);

    let mut window = Window::new_with_attributes(window_attributes)?;

    while window.pull() {
    
    }

    Ok(())

}
