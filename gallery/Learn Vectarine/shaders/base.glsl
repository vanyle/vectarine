// 100x100
// We try to match syntax from https://www.shadertoy.com/, with almost 100% compatibility

// If we see that one of these uniforms is used, we provide it
/*
uniform vec3      iResolution;           // viewport resolution (in pixels)
uniform float     iTime;                 // shader playback time (in seconds)
uniform float     iTimeDelta;            // render time (in seconds)
uniform float     iFrameRate;            // shader frame rate
uniform int       iFrame;                // shader playback frame
uniform float     iChannelTime[4];       // channel playback time (in seconds)
uniform vec3      iChannelResolution[4]; // channel resolution (in pixels)
uniform vec4      iMouse;                // mouse pixel coords. xy: current (if MLB down), zw: click
uniform samplerXX iChannel0..3;          // input channel. XX = 2D/Cube
uniform vec4      iDate;  
*/
// We also expect a mainImage entry point that we will call from our own main.

out vec4 fragColor;
in vec2 uv;
void main(){
    mainImage(fragColor, uv);
}

void mainImage( out vec4 o, in vec2 uv ){
    // Generate a red-green gradient
    o = vec4(uv.x,uv.y,0,1);
}