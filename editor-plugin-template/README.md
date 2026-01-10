# Vectarine Plugins

This is a template for a Vectarine plugin.

You can use it to expand the features of Vectarine using native (Rust) code.

## Getting started

## Using your plugin

Vectarine comes bundled with a runtime that is precompiled for all the major platforms.
For your plugins, you will need to manually compile them for the platforms you want to support.

If you don't have a Mac, you won't be able to support MacOS users for example.

## Distributing your plugin

Vectarine can load plugins in 2 ways:

- From a local file
- From the plugin registry

Plugins can have 2 formats:

- Unpackage plugins are the main format for plugin developpers, it is just the path to the shared library with the plugin
- Package plugins are the format for end users, a zip file with the shared libraries with the different platforms an documentation.

If your plugin extends the Lua, you should provide a `luau-api` folder to document the APIs you provide as well as a `README.md` file
for a description of your plugin, an **examples** on how to use it.
