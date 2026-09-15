use std::{
    f32::consts::{FRAC_PI_6, PI},
    ops::Mul,
    u32,
};

use bytemuck::{Pod, Zeroable, cast_slice};
use itertools::Itertools;
use wgpu::{
    Buffer, BufferUsages, Device, IndexFormat, PrimitiveState, PrimitiveTopology, VertexAttribute,
    VertexBufferLayout, VertexFormat,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::util::linspace;

/// see the following for benhcmarking of linear algebra crates:
/// https://github.com/bitshifter/mathbench-rs
/// https://marctenbosch.com/quaternions/ rotors are worth checking out as an alternative to quarternions,
/// and are part of ultraviolet
/// glam tends to perform best for single ops, and ultraviolet for batched ops

pub type Vertex = glam::f32::Vec4;

pub struct MeshDataDescriptor {}

impl MeshDataDescriptor {
    pub const ATTRIBUTE: VertexAttribute = VertexAttribute {
        format: VertexFormat::Float32x4,
        offset: 0,
        shader_location: 0, //TODO static allocate this value
    };
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[Self::ATTRIBUTE],
    };

    pub const INDEX_FORMAT: IndexFormat = IndexFormat::Uint32;

    pub const PRIMITIVE_STATE: PrimitiveState = PrimitiveState {
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        //cull_mode: None,
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        //polygon_mode: wgpu::PolygonMode::Line, //wireframe
        conservative: false,
    };
}

/// remember to specify indices in ccw order.
#[derive(Zeroable, Clone, Copy, Pod, Debug)]
#[repr(C)]
pub struct TriangleIdxSet(u32, u32, u32);

impl Mul<u32> for TriangleIdxSet {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

#[derive(Debug)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<TriangleIdxSet>,
}

#[derive(Debug)]
pub struct MeshAllocation {
    pub vtx_buffer: Buffer,
    pub vtx_count: u32,
    pub idx_buffer: Buffer,
    pub idx_count: u32,
}

impl MeshAllocation {
    pub fn new(device: &Device, mesh: &Mesh) -> Self {
        assert!(mesh.vertices.len() < u32::MAX as usize);
        assert!(mesh.indices.len() < u32::MAX as usize);
        Self {
            vtx_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: cast_slice(&mesh.vertices),
                usage: BufferUsages::VERTEX,
            }),
            vtx_count: mesh.vertices.len() as u32,
            idx_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: cast_slice(&mesh.indices),
                usage: BufferUsages::INDEX,
            }),
            idx_count: (mesh.indices.len() * 3) as u32,
        }
    }

    //TODO: merging multiple meshes into one allocation
}

//TODO z depth is 0-1 so need to map in projection matrix

/// The Box. You opened it. We came.
pub fn cube() -> Mesh {
    Mesh {
        vertices: vec![
            //front face, ccw
            Vertex::new(-1.0, 1.0, -1.0, 1.0),
            Vertex::new(-1.0, -1.0, -1.0, 1.0),
            Vertex::new(1.0, -1.0, -1.0, 1.0),
            Vertex::new(1.0, 1.0, -1.0, 1.0),
            //back face cw
            Vertex::new(-1.0, 1.0, 1.0, 1.0),
            Vertex::new(-1.0, -1.0, 1.0, 1.0),
            Vertex::new(1.0, -1.0, 1.0, 1.0),
            Vertex::new(1.0, 1.0, 1.0, 1.0),
        ],
        indices: vec![
            //front
            TriangleIdxSet(0, 1, 2),
            TriangleIdxSet(0, 2, 3),
            //back
            TriangleIdxSet(6, 5, 4),
            TriangleIdxSet(7, 6, 4),
            //left
            TriangleIdxSet(0, 4, 1),
            TriangleIdxSet(1, 4, 5),
            //right
            TriangleIdxSet(2, 6, 3),
            TriangleIdxSet(3, 6, 7),
            //top
            TriangleIdxSet(0, 3, 4),
            TriangleIdxSet(4, 3, 7),
            //bottom
            TriangleIdxSet(1, 2, 5),
            TriangleIdxSet(2, 6, 5),
        ],
    }
}

/// for uniform surface subdivision, formed by pie slicing
pub fn polygonal_prism(sides: u32) -> Mesh {
    let sides = sides.clamp(3, u32::MAX);

    let vtx = linspace::<f32, f32>(0.0, 2.0 * PI, (sides + 1) as usize)
        .skip(1)
        .map(|a| {
            [
                Vertex::new(a.sin(), 1.0, a.cos(), 1.0),
                Vertex::new(a.sin(), -1.0, a.cos(), 1.0),
            ]
        });
    let mut vtx: Vec<_> = vtx.flatten().collect();

    let mut idx = Vec::with_capacity(vtx.len() - 2);

    (0u32..vtx.len() as u32)
        .tuple_windows::<(_, _, _, _)>()
        .step_by(2)
        .for_each(|x| {
            // triangle strip looks like this as we start from top vertex:
            // \/\/ ...
            // second triangle, need to rearrange points otherwise would not be ccw
            idx.push(TriangleIdxSet(x.0, x.1, x.2));
            idx.push(TriangleIdxSet(x.1, x.3, x.2));
        });
    idx.push(TriangleIdxSet(
        (vtx.len() - 2) as u32,
        (vtx.len() - 1) as u32,
        0,
    ));
    idx.push(TriangleIdxSet((vtx.len() - 1) as u32, 1u32, 0));

    //top and bottom points for center of circle
    vtx.push(Vertex::new(0.0, 1.0, 0.0, 1.0));
    vtx.push(Vertex::new(0.0, -1.0, 0.0, 1.0));

    let vl = vtx.len() as u32;

    //top  and bottom circles
    (0u32..idx.len() as u32)
        .step_by(2)
        .tuple_windows::<(_, _)>()
        .for_each(|x| {
            idx.push(TriangleIdxSet(vl - 2, x.0, x.1));
        });

    (1u32..idx.len() as u32)
        .step_by(2)
        .tuple_windows::<(_, _)>()
        .for_each(|x| {
            idx.push(TriangleIdxSet(vl - 1, x.0, x.1));
        });

    // TODO
    //assert!(idx.len() == vtx.len(), "mesh idx list size did not match expected" );

    Mesh {
        vertices: vtx,
        indices: idx,
    }
}

//we can do better by placing a triangle in the circle center then further circles on the edges

// /// for uniform surface subdivision. formed by face subdivision of icosahedron
pub fn icosahedron() -> Mesh {
    //first generate isocahedron points:
    let mut vertices: Vec<Vertex> = Vec::with_capacity(12);
    //top
    vertices.push(Vertex::new(0.0, 1.0, 0.0, 1.0));
    //top disc
    for i in 0..5 {
        vertices.push(Vertex::new(
            f32::sin(PI * 2.0 * (i as f32) / 5.0),
            1.0 / 5.0_f32.sqrt(),
            f32::cos(PI * 2.0 * (i as f32) / 5.0),
            1.0,
        ));
    }
    //bottom disc
    for i in 0..5 {
        vertices.push(Vertex::new(
            f32::sin((PI * 2.0 * (i as f32) / 5.0) + FRAC_PI_6),
            -1.0 / 5.0_f32.sqrt(),
            f32::cos((PI * 2.0 * (i as f32) / 5.0) + FRAC_PI_6),
            1.0,
        ));
    }
    //bottom
    vertices.push(Vertex::new(0.0, -1.0, 0.0, 1.0));
    Mesh {
        vertices: vertices,
        indices: vec![
            TriangleIdxSet(0, 1, 2),
            TriangleIdxSet(0, 2, 3),
            TriangleIdxSet(0, 3, 4),
            TriangleIdxSet(0, 4, 5),
            TriangleIdxSet(0, 5, 1),
            //0
            //1  2  3  4  5
            //  6  7  8  9  10
            //11
            TriangleIdxSet(1, 6, 2),
            TriangleIdxSet(6, 7, 2),
            TriangleIdxSet(2, 7, 3),
            TriangleIdxSet(7, 8, 3),
            TriangleIdxSet(3, 8, 4),
            TriangleIdxSet(8, 9, 4),
            TriangleIdxSet(4, 9, 5),
            TriangleIdxSet(9, 10, 5),
            TriangleIdxSet(5, 10, 1),
            TriangleIdxSet(10, 6, 1),
            TriangleIdxSet(11, 7, 6),
            TriangleIdxSet(11, 8, 7),
            TriangleIdxSet(11, 9, 8),
            TriangleIdxSet(11, 10, 9),
            TriangleIdxSet(11, 6, 10),
        ],
    }
}

pub fn subdivide_mesh(mesh: Mesh, subdivision_level: u32) -> Mesh {
    if subdivision_level == 0 {
        return mesh;
    }

    //given a triangle, we can subdivide each side into n+1 points / n segments
    //(subdivision level 0 leaves unmodified)
    //e.g. for subdivision level 4:
    //    /\
    //   /\/\
    //  /\/\/\
    // /\/\/\/\
    // the bottom strip contains 1 + 2n triangles
    // (n+1)^2 triangles total
    // number of vertexes is (n+3)(n+2)/2

    let _nv = mesh.vertices.len();
    let new_vtx_count = (subdivision_level + 3) * (subdivision_level + 2) / 2;
    let new_idx_count = (subdivision_level + 1).pow(2);
    let mut new_vertices: Vec<Vertex> = Vec::with_capacity(new_vtx_count as usize);
    let mut new_indexes: Vec<TriangleIdxSet> = Vec::with_capacity(new_idx_count as usize);

    for idx_set in mesh.indices {
        //points that split the left and right sides of the triangle
        let lh_iter = linspace::<f32, _>(
            mesh.vertices[idx_set.0 as usize],
            mesh.vertices[idx_set.1 as usize],
            (subdivision_level + 2) as usize,
        );
        let rh_iter = linspace::<f32, _>(
            mesh.vertices[idx_set.0 as usize],
            mesh.vertices[idx_set.2 as usize],
            (subdivision_level + 2) as usize,
        );

        let side_iter = lh_iter.zip(rh_iter);

        for (i, ((lhi_prev, rhi_prev), (lhi, rhi))) in
            side_iter.tuple_windows::<(_, _)>().enumerate()
        {
            if i == 0 {
                let vprev = new_vertices.len() as u32;
                new_vertices.extend([lhi_prev, lhi, rhi]); //lhi_prev == rhi_prev when i=0
                new_indexes.push(TriangleIdxSet(vprev, vprev + 1, vprev + 2));
                continue;
            }

            let mut tl_tr_strip_points = linspace::<f32, _>(lhi_prev, rhi_prev, i + 1);
            let mut bl_br_strip_points = linspace::<f32, _>(lhi, rhi, i + 2);

            for (i, (a, b, c)) in tl_tr_strip_points
                .by_ref()
                .zip(bl_br_strip_points.by_ref())
                .flat_map(|(a, b)| [b, a])
                .tuple_windows::<(_, _, _)>()
                .enumerate()
            {
                let vprev = new_vertices.len() as u32;

                new_vertices.extend([a, b, c]);

                if i % 2 == 0 {
                    new_indexes.push(TriangleIdxSet(vprev, vprev + 2, vprev + 1));
                } else {
                    // reverse needed to maintain for ccw
                    new_indexes.push(TriangleIdxSet(vprev, vprev + 1, vprev + 2));
                }
            }

            let vprev = new_vertices.len() as u32;
            new_vertices.push(bl_br_strip_points.next().unwrap());
            new_indexes.push(TriangleIdxSet(vprev, vprev - 1, vprev - 2));
        }
    }

    //TODO fails
    //assert!(new_vertices.len() as u32 == new_vtx_count,"{}",format!("mesh vtx count {}, not {}",new_vertices.len(),new_vtx_count));
    //assert!(new_indexes.len() as u32 == new_idx_count,"{}",format!("mesh idx count {}, not {}",new_indexes.len(),new_idx_count));

    Mesh {
        vertices: new_vertices,
        indices: new_indexes,
    }
}

#[test]
fn test_subdivide_triangle() {
    let m = subdivide_mesh(triangle(), 1);
    println!("{:?}", m);
}

///# Returns:
/// Sphere mesh of 12 * (2.pow(subdivision_level)) triangles
pub fn sphere(subdivision_level: u32) -> Mesh {
    let ico = icosahedron();
    if subdivision_level == 0 {
        return ico;
    }

    let mut mesh = subdivide_mesh(ico, subdivision_level);
    //raise points to the surface of the sphere

    //current radius for sphere this point sits on
    for vtx in &mut mesh.vertices {
        let r = (vtx.x.powi(2) + vtx.y.powi(2) + vtx.z.powi(2)).sqrt();
        vtx.x = vtx.x * (1.0 / r);
        vtx.y = vtx.y * (1.0 / r);
        vtx.z = vtx.z * (1.0 / r);
    }
    mesh
}

pub fn triangle() -> Mesh {
    Mesh {
        vertices: vec![
            Vertex::new(-1.0, -1.0, 0.0, 1.0),
            Vertex::new(1.0, -1.0, 0.0, 1.0),
            Vertex::new(0.0, 1.0, 0.0, 1.0),
        ],
        indices: vec![TriangleIdxSet(0, 1, 2)],
    }
}

//todo unit test
fn validate_mesh(_mesh: &Mesh) {
    //assert indexes are in valid range
    //check for repeated elements?
    //check all points are used
}

#[test]
fn validate_meshes() {
    validate_mesh(&cube());
    validate_mesh(&icosahedron());
    //validate_mesh(sphere());
    //validate_mesh(cylinder());
}

//TODO: clipping

#[derive(strum_macros::Display, strum_macros::EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum Geometries {
    Sphere,
    #[strum(to_string = "Polygonal Prism")]
    PolygonalPrism,
    Cube,
}

impl Geometries {
    //idk about the subdivision level parameter here, its not always applicable
    pub fn mesh(self, subdivision_level: u32) -> Mesh {
        match self {
            Self::Sphere => sphere(subdivision_level),
            Self::Cube => cube(),
            Self::PolygonalPrism => polygonal_prism(subdivision_level),
        }
    }
}  