use crate::entity::entity::TexturedVertex;

pub struct TexturedCube {
    pub vertices: Vec<TexturedVertex>,
    pub indices: Vec<u16>,
}
const VERTICES: &[TexturedVertex] = &[
    TexturedVertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    }, // A
    TexturedVertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [0.0, 1.0],
    }, // B
    TexturedVertex {
        position: [0.0, 1.0, 1.0],
        tex_coords: [0.0, 1.0],
    }, // C
    TexturedVertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [0.0, 0.0],
    }, // D
    TexturedVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    }, // A
    TexturedVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
    }, // B
    TexturedVertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [1.0, 1.0],
    }, // C
    TexturedVertex {
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
impl TexturedCube {
    pub fn new() -> TexturedCube {
        TexturedCube {
            vertices: VERTICES.to_vec(),
            indices: INDICES.to_vec(),
        }
    }
}

use crate::entity::entity::PrimitiveVertex;

pub struct PrimitiveCube {
    pub vertices: Vec<PrimitiveVertex>,
    pub indices: Vec<u16>,
}
const PRIMITIVE_VERTICES: &[PrimitiveVertex] = &[
    PrimitiveVertex {
        position: [0.0, 0.0, 1.0],
        color: [1.0, 0.0, 1.0],
    }, // A
    PrimitiveVertex {
        position: [1.0, 0.0, 1.0],
        color: [1.0, 0.0, 1.0],
    }, // B
    PrimitiveVertex {
        position: [0.0, 1.0, 1.0],
        color: [1.0, 0.0, 1.0],
    }, // C
    PrimitiveVertex {
        position: [1.0, 1.0, 1.0],
        color: [1.0, 0.0, 1.0],
    }, // D
    PrimitiveVertex {
        position: [0.0, 0.0, 0.0],
        color: [1.0, 0.0, 1.0],
    }, // A
    PrimitiveVertex {
        position: [1.0, 0.0, 0.0],
        color: [1.0, 0.0, 1.0],
    }, // B
    PrimitiveVertex {
        position: [0.0, 1.0, 0.0],
        color: [1.0, 0.0, 1.0],
    }, // C
    PrimitiveVertex {
        position: [1.0, 1.0, 0.0],
        color: [1.0, 0.0, 1.0],
    }, // D
];
impl PrimitiveCube {
    pub fn new() -> PrimitiveCube {
        PrimitiveCube {
            vertices: PRIMITIVE_VERTICES.to_vec(),
            indices: INDICES.to_vec(),
        }
    }
}
