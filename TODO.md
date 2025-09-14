# TODO

- [ ] Loading fonts and drawing text from lua code
- [ ] Allow window resizing and handle aspect ratios
- [ ] Fullscreen API (for web too!)
- [ ] More complete drawing API
  - [ ] Draw portions of images
  - [ ] Rotation
  - [ ] Lines
  - [ ] Quads
  - [ ] Polygons
  - [ ] Outlines
- [ ] Canvas API
- [ ] 'Screen' system (Menu, Settings, Game, Pause...)
- [ ] Tiled parsing
- [ ] Textures as an asset class
- [ ] Investigate how to have true interactive documentation

# Done

- [x] Assets "resource" system
  - [x] Resource tab in editor
  - [x] Error system when asset fails to load
  - [x] Hot reloading for assets
- [x] Loading images and drawing them from lua code
- [x] Performance: Batch drawing implementation
- [x] Window size
- [x] Mouse input
- [x] Editor console
- [x] pthread support (requires rust nightly, but it works)
- [x] Setup the editor
  - [x] When no CLI arguments are passed, open the editor
  - [x] Show EGui widget in the editor
  - [x] Use OpenGL for quad drawing in the runtime
  - [x] Be able to load a game.lua file
- [x] Cross-platform keyboard input
  - [x] Access to keyboard from lua
- [x] Loading external files (like lua) from rust, for web.
  - [x] Call JS from rust
  - [x] Call rust from JS
  - [x] Calling async function from both platforms
  - [x] Reading a lua file
- [x] Properly manage the different build types (using cargo workspaces probably?)
  - [x] Make web build and native build work without config changes
- [x] Web build
- [x] Luau : hot reload for script
- [x] Luau (with a function to draw rects probably)
