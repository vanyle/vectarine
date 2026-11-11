# How does Vectarine draw?

> This documentation is for engine developers, not game builders!

Drawing is done with [wgpu](https://wgpu.rs/), which uses Metal on macOS, DirectX 12 or Vulkan on Windows, Vulkan on
Linux and WebGL 2 on the web (OpenGL is the fallback on Windows and Linux). The code is in `runtime/src/graphics`.

