#version 400 core

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 color;

struct Complex {
	double real;
	double imag;
};

uniform vec2 resolution;
uniform int max_iter;

uniform vec3 cam;

uniform float animation;

const int KIND_MANDELBROT = 0;
const int KIND_JULIA = 1;
uniform int kind;

int mandelbrot(in Complex c) {
	Complex z = c;
	for (int i = 0; i < max_iter; i++) {
		if (z.real*z.imag >= 4.0) { // euclidian distance >= 2
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
		if (z.real*z.imag >= 4.0) { // euclidian distance >= 2
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

	dvec3 dcam = dvec3(cam) / 10000.0;

	Complex z = Complex(
		double(p.x - 0.5) * dcam.z + dcam.x,
		double((p.y - 0.5) * resolution.y/resolution.x) * dcam.z + dcam.y
	);

	int i = 0;
	if (kind == KIND_MANDELBROT) i = mandelbrot(z);
	else i = julia(z, Complex(double(cos(animation)), double(sin(animation))));

	if (i == max_iter) color = vec4(0.0, 0.0, 0.0, 1.0);
	else {
		float value = float(i) / float(max_iter);
		color = vec4(1.0 - value, 0.5 - value*0.5, 0.5, 1.0);
	}
}
