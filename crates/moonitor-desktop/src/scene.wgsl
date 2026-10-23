struct Uniforms { camera: mat4x4<f32>, light: mat4x4<f32>, eye:vec4<f32>, state:vec4<f32>, nodes:array<vec4<f32>,32> }
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var shadow_map:texture_depth_2d;
@group(0) @binding(2) var shadow_sampler:sampler_comparison;
struct Input { @location(0) position:vec3<f32>, @location(1) normal:vec3<f32>, @location(2) color:vec4<f32>, @location(3) material:vec4<f32> }
struct Output { @builtin(position) position:vec4<f32>, @location(0) world:vec3<f32>, @location(1) normal:vec3<f32>, @location(2) color:vec4<f32>, @location(3) material:vec4<f32>, @location(4) shadow:vec4<f32> }
@vertex fn vs_main(v:Input)->Output {
    var o:Output; o.world=v.position; o.position=u.camera*vec4(v.position,1.0);o.normal=v.normal;o.color=v.color;o.material=v.material;o.shadow=u.light*vec4(v.position,1.0); return o;
}
@vertex fn vs_shadow(v:Input)->@builtin(position) vec4<f32> {return u.light*vec4(v.position,1.0);}
@fragment fn fs_main(v:Output)->@location(0) vec4<f32> {
    let n=normalize(v.normal);let l=normalize(vec3(-3.0,9.0,5.0));let eye=normalize(u.eye.xyz-v.world);
    let h=normalize(l+eye);let rough=v.material.y;
    let s=v.shadow.xyz/v.shadow.w;let uv=vec2(s.x*0.5+0.5,0.5-s.y*0.5);
    var shade=0.0;
    for(var x=-1;x<=1;x++){for(var y=-1;y<=1;y++){shade+=textureSampleCompare(shadow_map,shadow_sampler,uv+vec2(f32(x),f32(y))/1024.0,s.z-0.0015);}}
    shade=shade/9.0;
    let diffuse=max(dot(n,l),0.0);
    let hemi=0.36+0.30*(n.y*0.5+0.5);
    let spec=pow(max(dot(n,h),0.0),mix(120.0,12.0,rough))*(1.0-rough)*0.4;
    var base=pow(v.color.rgb,vec3(2.2));
    if(v.material.x>=0.0) {
        let node=u.nodes[min(u32(v.material.x),31u)];
        if(v.material.z==2.0 || v.material.z==1.0) {
            base=pow(mix(node.rgb,vec3(0.82,0.91,0.85),node.a*0.35),vec3(2.2));
        }
    }
    var color=base*(hemi+diffuse*shade*0.48)+vec3(spec*shade);
    if(v.material.z==1.0) { color=base*0.85; }
    if(v.material.z==3.0) {
        // A quiet ground plane with real device shadows, no perpetual animation.
        color=pow(mix(vec3(0.95,0.956,0.938),vec3(0.09,0.14,0.11),u.state.x),vec3(2.2))*(0.89+0.11*shade);
    }
    return vec4(color,1.0);
}
