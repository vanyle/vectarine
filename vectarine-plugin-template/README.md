# Vectarine Plugins

This is a template for a Vectarine plugin.

You can use it to expand the features of Vectarine using native (Rust) code.

## Getting started

You can build your plugin using the `bundle.py` script.
Run it using `uv run bundle.py`.

You'll need to have [uv](https://github.com/astral-sh/uv) installed.

This will produce a `plugin.vectaplugin` file that you can copy to the `plugins` folder of the editor to use it.

## Distributing your plugin

To share your plugin, simply share the `your_plugin_name.vectaplugin` file.

## Platform support

Vectarine comes bundled with a runtime that is precompiled for all the major platforms.
However, as plugins contain native code, you will need to manually compile them for the platforms you want to support.

If you don't compile a Mac version of your plugin, games using it won't be able to run on Mac.

## Lua API

If your plugin extends the Lua, you should provide inside `plugin.luau` a list of the function you define, with documentation comments and proper types.

## Plugin capabilities

As vectarine plugins are written in Rust, they can do pretty much anything, as long as the platform they are compiled for supports it.

Moreover, plugins use the SDK as a dependency to be able to access and create Luau functions for users.
