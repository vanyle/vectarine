# Game structure

Projects can have 2 forms:

- The editable form is opened by the editor
- The runtime form is make to be ran by the runtime

You can turn the editable form into the runtime form but not the other way around

## Editable form

This can be either a folder or a ZIP file. It recommend working with a folder, but the editor can open and edit ZIPs.
Zips can be used to quickly share and open projects between developers.
The runtime is also able to run a project in folder or zip for quick testing.

Editable projects have the following file structure:

```
├── game.vecta
├── fonts
├── scripts/
│   └── game.luau
├── textures
└── shaders
```

`game.vecta` is a toml file containing metadata about the game like:

- The name of the game
- The logo to use for the splash screen
- Tags
- A description
- The default window size / fullscreen

## Runtime form

The runtime form is optimized for performance, small size and fast loading.
The `luau` compiler is used to compiler the different lua chunks
The assets are processed using a pipeline system so that everything is optimized and loads quickly.

Everything is put into one file compressed with zstd and named `bundle.vecta` too. Encryption can optionally be added to make reverse engineering slighltly harder.

```
├── bundle.vecta
└── game.exe
```

## Pipeline

Feature ideas

- Custom pipeline plugins
- Interactions between the pipeline and modding support
- Process wav, mp3, etc... into one good format (probably a mix of lossy/lossless?)
- Process all images, tilemaps, etc... into pngs
- Process all xml/json style files (tmx, etc...) into binary blobs
