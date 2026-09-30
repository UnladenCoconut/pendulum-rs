#include "render-binds.slang"

[shader("vertex")]
VertexOutput vs_main(IndexInput iin) {
    VertexOutput out;

    VertexInput vin = data.verticies[iin.index];

    out.position = mul(data.transforms.camera, mul(data.transforms.object, vin.position));
    out.color = float4((out.position[0]+1.0) / 2.0, (out.position[1]+1.0) / 2.0, (out.position[2] + 1.0) / 2.0, 1.0);
    return out;
}

struct FragOutput {
    float4 color : SV_Target0;
};

[shader("fragment")]
FragOutput fs_main(VertexOutput v) {
    FragOutput out;
    out.color = v.color;
    return out;
}