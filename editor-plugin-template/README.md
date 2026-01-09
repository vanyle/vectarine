# Vectarine Plugins

This is a template for a Vectarine plugin.

You can use it to expand the features of Vectarine using native (Rust) code.

## Getting started

## Using your plugin

Vectarine comes bundled with a runtime that is precompiled for all the major platforms.
For your plugins, you will need to manually compile them for the platforms you want to support.

If you don't have a Mac, you won't be able to support MacOS users for example.

## Distributing your plugin

You can share plugins as vectarine is able to load them from a (git) URL.

If your plugin extends the Lua, you should provide a `luau-api` folder to document the APIs you provide as well as a `README.md` file
for a description of your plugin, an **examples** on how to use it.
