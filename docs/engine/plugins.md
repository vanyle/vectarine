# Plugins

Vectarine Plugins are programs that extend the capabilities of Vectarine. 
Plugins can be written in Rust or any native language.

## What is a plugin exactly?

Plugins are zip files with the ".vectaplugin" extension.
A valid plugin contains a manifest.toml file with its name, a version number, a url and a description.

The url can be used to point to documentation and is shown in the plugin list.

In addition to that, for every supported platform, there is a folder in the zip containing the associated native code.

All in all, for a plugin supporting Window, Linux, Mac OS and the web, the zip structure looks like so:

```
plugin.zip
|- manifest.toml 
|- plugin.luau
|- windows/
    |- plugin.dll
|- linux/
    |- plugin.so
|- macos/
    |- plugin.dylib
|- web/
    |- plugin.wasm
```

## Creating a plugin

Plugins can be made using the `vectarin-plugin-template` which contains a script that generates a sample `example.vectaplugin` from a rust project.

## Plugins capabilities

Plugins can execute any native code, so they can do pretty much anything without any sandboxing. They are able to extend the lua environment by adding new functions and
call existing ones. They use hooks to inject code when the game is loaded, on before and after every frame and on game shutdown.

If a plugin adds to the Lua API, it is good practice to ship with it a `plugin.luau` file. This plugin will get automatically copied to the `luau-api` file of the game to
provide the user with autocompletion.

> Important note: Currently, plugins are purely code and cannot include extra assets.
> If you need to do so, you need to use `include_bytes!` and ship your data directly inside the native code.

## In the editor

The editor is shipped with a plugins folder. This folder is empty, but new plugins can be added inside so that they are "made available" to the editor.
Plugins inside the editor folder are called "trusted plugins".

Every game can also have a "plugins" folder. From the plugin interface in the editor, you can copy plugins file from the editor to the game.
Only plugins in the game that have the same hash as trusted plugins are executed.

This prevents people from getting viruses when opening random projects.
If a game uses untrusted plugins, a warning label in the menu bar of the editor appear to explain the issue. You can choose to trust a plugin by copying it
to the trusted plugins folder.

When the list of plugins of the game changes, the game is restarted to have a coherent state.

## In the game

When the game is distributed, only the relevant native code is shipped in the `.vecta` package. As start-up, the game is able to extract the .dlls/.so/.dynlib from the zip to execute them.
This is because you cannot execute dlls from memory, they need to be in file form at some point, and we use the lowest common denominator between platforms.
