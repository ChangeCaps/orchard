struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] index: u32) -> VertexOutput {
	var out: VertexOutput;	

	let x = -1.0 + f32((index & 1u32) << 2u32);
	let y = -1.0 + f32((index & 2u32) << 1u32);
	out.uv.x = (x + 1.0) * 0.5;
	out.uv.y = 1.0 - (y + 1.0) * 0.5;

	out.position = vec4<f32>(x, y, 0.0, 1.0);

	return out;
}

[[group(0), binding(0)]]
var texture: texture_2d<f32>;

[[group(0), binding(1)]]
var depth: texture_depth_2d;

[[group(0), binding(2)]]
var sampler: sampler;

struct FragmentOutput {
	[[location(0)]] color: vec4<f32>;
	[[builtin(frag_depth)]] depth: f32;
};

[[stage(fragment)]]
fn main(in: VertexOutput) -> FragmentOutput {
	var out: FragmentOutput;
	
	out.color = textureSample(texture, sampler, in.uv);
	out.depth = textureSample(depth, sampler, in.uv);

	return out;
}