#version 400 core

in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 color;

uniform vec2 resolution;
uniform int max_iter;

uniform vec3 cam;

struct Complex {
	double real;
	double imag;
};

int mandelbrot(in Complex c) {
	Complex z = Complex(0.0, 0.0);

	for (int i = 0; i < max_iter; i++) {
		if (z.real >= 2.0 || z.imag >= 2.0) {
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
		double(p.x - 0.5) * double(cam.z) + double(cam.x),
		double(p.y - 0.5) * double(resolution.y/resolution.x) * double(cam.z) + double(cam.y)
	);

	int i = mandelbrot(z);
	if (i == max_iter) color = vec4(0.0, 0.0, 0.0, 1.0);
	else {
		float value = float(i) / float(max_iter);
		color = vec4(1.0 - value, 0.5 - value*0.5, 0.5, 1.0);
	}
}
