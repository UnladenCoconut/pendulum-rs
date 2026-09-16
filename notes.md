indexed mesh/es

scene
    objects
        meshes

n objects, with object transform in world space

m parts in object (4 for pendulum), made of o meshes

so indexed meshes, per-part transforms in object space 


- vertex and indicies buffers
- meshes denoted by contiguous? index range. indexes per mesh have to be offset by their position?
so mesh id -> vertex id offset 


each object has o unique mesh transforms for each part
- object buffer of object transforms
- mesh instance transform buffer - object id transform idx (for object buffer ), object-space part transform
 
### Wireframes 
PolygonMode::Line doesnt work for the webgpu backend.
There are a few ways we can draw wireframes:
- Draw meshes as PrimitiveType::LineList, after generating a new indicies buffer
- Use barycentric coordinates and a wireframe shader. WGpu does actually have `wgpu::Features::SHADER_BARYCENTRIC_COORDINATES` but this is only for native targets.
    - SV_VertexID could be used in a shader to generate barycentric coords for non-indexed rendering. With indexed rendering we aren't garunteed to ge t `SV_VertexID % 3 = {0,1,2}` for all verticies in the triangle as they arent neccesarilly sequential.
    - For indexed rendering, we need to use our indices buffer as the shader's vertex buffer, and pass in out vertex data array as a uniform to the shader that we can retrieve the vertex from. SV_VertexID can then be used reliably. 