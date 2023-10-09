#version 330 core

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 color;

struct Complex {
	float real;
	float imag;
};

uniform vec2 resolution;
uniform int max_iter;

uniform vec3 cam;

uniform vec2 animation;

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

	Complex z = Complex(
		(p.x - 0.5) * cam.z + cam.x,
		((p.y - 0.5) * resolution.y/resolution.x) * cam.z + cam.y
	);

	int i = 0;
	if (kind == KIND_MANDELBROT) i = mandelbrot(z);
	else {
		i = julia(z, Complex(animation.x, animation.y));
	}

	const vec3 colors[6] = vec3[6](
		vec3(0.1, 0.1, 0.3),
		vec3(0.1, 0.2, 0.9),
		vec3(0.1, 0.95, 0.95),
		vec3(0.2, 0.95, 0.1),
		vec3(0.9, 0.95, 0.1),
		vec3(0.95, 0.1, 0.1)
	);

	if (i == max_iter) color = vec4(0.0, 0.0, 0.0, 1.0);
	else {
		float range = float(i) / float(max_iter);
		range = 1.0 - range;
		range = 1.0 - pow(range, 3); // spread the colors more evenly
		range *= 5;

		vec3 a = colors[uint(floor(range))];
		vec3 b = colors[uint(ceil(range))];
		color = vec4(mix(a, b, fract(range)), 1.0);
	}
}
