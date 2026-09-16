/*
shader recieves :
renderer
shader source / name
scene resource Data (atm meshes)

shader does:
loading shader 
defining bind info for shader (hardcoded, but could use shader slang)



*/

use wgpu::*;


pub const PRIMITIVE_STATE: PrimitiveState = PrimitiveState {
    topology: PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: wgpu::FrontFace::Ccw,
    cull_mode: Some(wgpu::Face::Back),
    //cull_mode: None,
    unclipped_depth: false,
    polygon_mode: wgpu::PolygonMode::Fill,
    //polygon_mode: wgpu::PolygonMode::Line, //wireframe, not supported on webgpu target
    conservative: false,
};

pub mod wireframe {
    use wgpu::*;
    use crate::geometry::TriangleIdxSet;

    pub const ATTRIBUTES: [VertexAttribute;1] = [
        VertexAttribute {
            format: VertexFormat::Uint32,
            offset: 0,
            shader_location: 0
        },
    ];

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
            array_stride: size_of::<TriangleIdxSet>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES
    };
}

pub mod fill_color {
    use wgpu::*;
    use crate::geometry::{Vertex,TriangleIdxSet};

    pub const ATTRIBUTES: [VertexAttribute;1] = [
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0, //TODO static allocate this value
        },
    ];

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &ATTRIBUTES,
    };

    pub const INDEX_FORMAT: IndexFormat = IndexFormat::Uint32;

    
}