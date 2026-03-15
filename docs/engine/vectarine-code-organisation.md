# How is vectarine's code organized?

Vectarine is made of 4 rust crates:

- runtime
- editor
- vectarine-plugin-template
- vectarine-plugin-sdk

We have the following static dependencies (--> = is used by):

vectarine-plugin-sdk --> runtime --> editor
                  \----> vectarine-plugin-template

## Who does what? What are the main structs?

**The runtime**

The runtime is responsible for running the packaged game. To avoid differences between the editor and the runtime execution, the editor
uses the runtime as a dependency to execute games.

The main object that the runtime uses is called `Game` and represents a game. There is only one game built during the life of the runtime for packaged games but this is not true inside the editor.

The runtime also setups the environment needed to run games. This includes the default Lua APIs, the resource system or the OpenGL function calls.

**The editor**

The editor is a GUI program that is able to run multiple games during its lifetime as well as debug them and provide hot reload.
The editor wraps the `Game` inside a `ProjectState` which is the combination of a `Game` with various infos the editor need to provide debugging,
and manage the associated project.

**The SDK**

The SDK is there to store types that need to be shared between plugins, the runtime and the editor. It is used to ensure that all these project use libraries of the same
version for ABI compatibility. It mostly contains struct and few functions.

**The plugin template**

The plugin template is a project that compiles to dynamic libraries to be stored inside a `.vectaplugin` file. This library is loaded by the runtime and editor to provide plugin capabilities.

## If I add a dependency, where should I add it?

If it exist only for **building** games (and not executing them), it should be inside the editor.

Example: toml_edit is used to edit toml files. This is only used to save game projects, so it is inside the editor.

If it is needed for **general game purposes**, it needs to be inside vectarine-plugin-sdk.

Example: rapier2d the physics library can be used by plugins and is needed to run games properly, so the runtime and editor need it.

If it is needed only for **loading a packaged game bundle**, it needs to be inside the runtime.

Example: emscripten-val is used for bridging the gap between rust code in the browser and JS. As the browser is a runtime
specific environment, this dependency is only in the runtime.

That's it! This plugin template has no extra dependency other than the runtime (but plugin creators can add extra dependencies if they want to.)
