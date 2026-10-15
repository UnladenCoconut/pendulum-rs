#include "render-binds.slang"

//use barycentric coords to draw lines

[shader("vertex")]
VertexOutput vs_main(IndexInput iin) {

    VertexOutput out;

    VertexInput vin = data.verticies[iin.index];

    out.barycentric = barycentric(iin.ID);
    
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

    float uh = max(max(ddx(v.barycentric.x),ddy(v.barycentric.y)),0.01); // div 0 errors?
    float line_thickness = uh;

    //float uh = 1.0;
    //float line_thickness = 0.01;

    float minBarry = min(v.barycentric.x, min(v.barycentric.y, v.barycentric.z));

    //out.color = v.color;

    out.color = lerp(v.color, float4(0.0, 0.0, 0.0, 1.0), smoothstep(0.0, line_thickness, minBarry));

    return out;
}