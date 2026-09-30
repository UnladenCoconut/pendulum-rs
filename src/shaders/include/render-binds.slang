
/*
to do proper rendering of wireframes wholly in the shader, whilst still using indexed verticies draw,
    indexes are passed via the vertex shader attributes, the real vertexes are indexed from a uniform buffer.
    we have to do is this way to be able to properly assign a unique ID to each vertex corner - we 
    have SV_VertexID but if we pass in verticies the normal way, we don't neccesarily get contigous ID's or unique set of all values {1,2,3} in mod 3. 

    we'll do this in all shaders for consistency and if we want to use barycentric coords later for anything else.
*/

struct Transforms {
    ConstantBuffer<matrix<float, 4, 4>> camera;
    ConstantBuffer<matrix<float, 4, 4>> object;
}

struct VertexInput {
    float4 position;
}

struct IndexInput {
    uint index;
    uint ID: SV_VertexID;
}

struct Data {
    StructuredBuffer<VertexInput> verticies;
    Transforms transforms;
}

ParameterBlock<Data> data;

struct VertexOutput {
    float4 position : SV_Position;
    float4 color;
    float3 barycentric;
};

float3 barycentric(uint id) {
    if (id % 3 == 0) {
        return float3(1.0, 0.0, 0.0);
    } else if (id % 3 == 1) {
        return float3(0.0, 1.0, 0.0);
    } else {
        return float3(0.0, 0.0, 1.0);
    }
}