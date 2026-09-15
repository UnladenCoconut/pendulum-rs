struct Transforms {
    ConstantBuffer<matrix<float, 4, 4>> camera;
    ConstantBuffer<matrix<float, 4, 4>> object;
}

ParameterBlock<Transforms> transforms;

struct VertexInput {
    float4 position;
    uint ID : SV_VertexID;
}

struct VertexOutput {
    float4 position : SV_Position;
    float4 color;
    float3 barycentric;
};

//use barycentric coords to draw lines

[shader("vertex")]
VertexOutput vs_main(VertexInput vin) {

    VertexOutput out;

    if (vin.ID % 3 == 0) {
        out.barycentric = float3(1.0, 0.0, 0.0);
    } else if (vin.ID % 3 == 1) {
        out.barycentric = float3(0.0, 1.0, 0.0);
    } else {
        out.barycentric = float3(0.0, 0.0, 1.0);
    }
    
    out.position = mul(transforms.camera, mul(transforms.object, vin.position));
    out.color = float4((out.position[0]+1.0) / 2.0, (out.position[1]+1.0) / 2.0, (out.position[2] + 1.0) / 2.0, 1.0);
    return out;
}

struct FragOutput {
    float4 color : SV_Target0;
};

[shader("fragment")]
FragOutput fs_main(VertexOutput v) {
    FragOutput out;

    float minBarry = min(v.barycentric.x, min(v.barycentric.y, v.barycentric.z));

    //out.color = v.color;

    out.color = lerp(v.color, float4(0.0, 0.0, 0.0, 1.0), smoothstep(0.0, 0.01, minBarry));

    return out;
}