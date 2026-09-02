struct Transforms {
    ConstantBuffer<matrix<float, 4, 4>> camera;
    ConstantBuffer<matrix<float, 4, 4>> object;
}

ParameterBlock<Transforms> transforms;

struct VertexOutput {
    float4 position : SV_Position;
};

[shader("vertex")]
VertexOutput vs_main(float4 position_in) {
    VertexOutput out;
    out.position = mul(transforms.camera, mul(transforms.object, position_in));
    return out;
}

struct FragOutput {
    float4 color : SV_Target0;
};

[shader("fragment")]
FragOutput fs_main(VertexOutput v) {
    FragOutput out;
    out.color = float4(1.0, 0.0, 0.0, 1.0);
    return out;
}