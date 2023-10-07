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

	Complex z = Complex(
		double(p.x - 0.5) * double(cam.z) + double(cam.x),
		double(p.y - 0.5) * double(resolution.y/resolution.x) * double(cam.z) + double(cam.y)
	);

	int i = julia(z, Complex(double(cos(animation)), double(sin(animation))));
	if (i == max_iter) color = vec4(0.0, 0.0, 0.0, 1.0);
	else {
		float value = float(i) / float(max_iter);
		color = vec4(1.0 - value, 0.5 - value*0.5, 0.5, 1.0);
	}
}
