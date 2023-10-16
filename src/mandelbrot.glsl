#version 330 core
#extension GL_ARB_gpu_shader_fp64 : enable

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 color;

struct Complex {
	double real;
	double imag;
};

uniform vec2 resolution;
uniform int max_iter;

uniform vec3 cam_big;
uniform vec3 cam_small;

uniform vec2 animation_big;
uniform vec2 animation_small;

const int KIND_MANDELBROT = 0;
const int KIND_JULIA = 1;
uniform int kind;

int mandelbrot(in Complex c) {
	Complex z = c;
	for (int i = 0; i < max_iter; i++) {
		if (z.real*z.real + z.imag*z.imag >= 4.0) { // euclidian distance >= 2
            return i;
       }

		// (a + bi)^2
		// a^2 + 2abi - b^2
		z = Complex(
			z.real*z.real - z.imag*z.imag,
			2.0*z.real*z.imag
		);

		z.real += c.real;
		z.imag += c.imag;
	}

    return max_iter;
}

int julia(in Complex z, in Complex c) {
	for (int i = 0; i < max_iter; i++) {
		if (z.real*z.real + z.imag*z.imag >= 4.0) { // euclidian distance >= 2
            return i;
       }

		// (a + bi)^2
		// a^2 + 2abi - b^2
		z = Complex(
			z.real*z.real - z.imag*z.imag,
			2.0*z.real*z.imag
		);

		z.real += c.real;
		z.imag += c.imag;
	}

    return max_iter;
}

void main() {
	vec2 p = gl_FragCoord.xy / resolution; 

	dvec3 dcam = dvec3(cam_big) + dvec3(cam_small);

	Complex z = Complex(
		double(p.x - 0.5) * dcam.z + dcam.x,
		double((p.y - 0.5) * resolution.y/resolution.x) * dcam.z + dcam.y
	);

	int i = 0;
	if (kind == KIND_MANDELBROT) i = mandelbrot(z);
	else {
		dvec2 animation = dvec2(animation_big) + dvec2(animation_small);
		i = julia(z, Complex(animation.x, animation.y));
	}

	// #define NUM_COLORS 6
	// const vec3 colors[NUM_COLORS] = vec3[NUM_COLORS](
	// 	vec3(0.1, 0.1, 0.3),
	// 	vec3(0.1, 0.2, 0.9),
	// 	vec3(0.1, 0.95, 0.95),
	// 	vec3(0.3, 0.8, 0.2),
	// 	vec3(0.9, 0.95, 0.1),
	// 	vec3(0.8, 0.2, 0.2)
	// );

	// #define NUM_COLORS 4
	// const vec3 colors[NUM_COLORS] = vec3[NUM_COLORS] (
	// 	vec3(0.349, 0.329, 0.341),  // #595457
	// 	vec3(0.619, 0.098, 0.274),  // #9E1946
	// 	vec3(0.871, 0.051, 0.572),  // #DE0D92
	// 	vec3(0.301, 0.423, 0.980)   // #4D6CFA
	// );

	#define NUM_COLORS 5
	const vec3 colors[NUM_COLORS] = vec3[NUM_COLORS] (
		vec3(0.588, 0.678, 0.784),  // #96ADC8
		vec3(0.843, 1.000, 0.671),  // #D7FFAB
		vec3(0.988, 1.000, 0.424),  // #FCFF6C
		vec3(0.847, 0.616, 0.416),  // #D89D6A
		vec3(0.427, 0.271, 0.298)   // #6D454C
	);

	if (i == max_iter) color = vec4(0.0, 0.0, 0.0, 1.0);
	else {
		float range = float(i*(NUM_COLORS+1)) / float(max_iter);

		vec3 a = colors[uint(floor(range))];
		vec3 b = colors[uint(ceil(range))];
		color = vec4(mix(a, b, fract(range)), 1.0);
	}
}
