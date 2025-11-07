precision mediump float;
in vec2 uv;
uniform sampler2D tex;
uniform float iTime;

uniform float lightZone; // size of the lighted area
uniform float lightPosX;
uniform float lightPosY;
uniform float windowWidth;
uniform float windowHeight;

out vec4 frag_color;

float calculateLightFalloff(float closeness, float lightZoneSize){
    return (1.1 * atan((closeness - (1.2 - lightZoneSize)) * 12.0) / 3.14159265) + 0.5;
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    // Convert to pixel coordinates for window-independent calculation
    vec2 pixelCoord = uv * vec2(windowWidth, windowHeight);
    vec2 lightPixelPos = vec2(lightPosX + 0.5, lightPosY + 0.5) * vec2(windowWidth, windowHeight);
    
    // Calculate distance in pixels, then normalize to fixed radius
    float pixelDistance = distance(pixelCoord, lightPixelPos);
    float normalizedDistance = pixelDistance / (lightZone * 700.0); // Fixed pixel radius
    float closenessToCenter = max(0.0, 1.0 - normalizedDistance);
    
    float lightMultiplier = calculateLightFalloff(closenessToCenter, lightZone);
    fragColor = texture(tex, uv) * vec4(lightMultiplier, lightMultiplier, lightMultiplier, 1.0);
}

void main() {
    mainImage(frag_color, uv);
}

