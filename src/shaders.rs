use wgpu::*;

use crate::geometry::TriangleIdxSet;


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

pub const ATTRIBUTES: [VertexAttribute;1] = [
    VertexAttribute {
        format: VertexFormat::Uint32,
        offset: 0,
        shader_location: 0
    },
];

pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<u32>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &ATTRIBUTES
};