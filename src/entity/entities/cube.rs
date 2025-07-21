use crate::entity::entity::Vertex;

pub struct Cube {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    }, // A
    Vertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [0.0, 1.0],
    }, // B
    Vertex {
        position: [0.0, 1.0, 1.0],
        tex_coords: [0.0, 1.0],
    }, // C
    Vertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [0.0, 0.0],
    }, // D
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    }, // A
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
    }, // B
    Vertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [1.0, 1.0],
    }, // C
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    }, // D
];
#[rustfmt::skip]
const INDICES: &[u16] = &[
    //
    7, 6, 2, 2, 3, 7, 
    //?
    0, 4, 5, 5, 1, 0, 
    0, 2, 6, 6, 4, 0, 
    //awd!
    7, 3, 1, 1, 5, 7, 
    //ss!
    3, 2, 0, 0, 1, 3, 
    //back!
    4, 6, 7, 7, 5, 4,
];
impl Cube {
    pub fn new() -> Cube {
        Cube {
            vertices: VERTICES.to_vec(),
            indices: INDICES.to_vec(),
        }
    }
}
