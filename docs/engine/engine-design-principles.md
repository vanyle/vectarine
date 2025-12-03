# Engine Design Principles

*A set of guidelines on how to design Vectarine*

## Our goals

Vectarine has 3 main goals, which are listed in its README.

- Making the game maker productive, not wasting its time.
- Empowering makers to create whatever they can imagine
- Run anywhere without lag

Most guidelines derive from these goals.

# Not wasting time

## It needs to work with hot reload

Vectarine's main feature is its instant hot reload for everything.
Quick feedback cycles are what make development enjoyable.

Thus, all features of the engine need to be built with this in mind. If something
does not work well with hotreload, it can be considered a bug.

## It needs to welcome first time users

When using the engine for the first time, a user needs to be able to quickly get going.
They shouldn't be confused about how to do something.

The starting screen has 2 main buttons, "Open Existing Project", which will be used to most and
"Create new project" which is the one for newcomers.

The engine is distributed with a pdf guide which is nicely formatted and details how to make games
as well as a gallery of examples.

## It should be fun to use!

Creating games is a hobby for many! So might as well have fun making them.
Using vectarine should be fun and enjoyable.

## Bug free renaming and refactors

Renaming elements and moving code should not introduce bugs and if it does,
these bugs should be caught by the type system.

## Discovering features should be easy

When wanting to do something new, you shouldn't need to watch a tutorial.
You can just use the autocompletion of the editor to find available functions.

The interface shows the tools available without overwhelming the user. Keyboard
shortcuts are written down to allow people to become power users.

# Empowering makers

## Providing low level and high level APIs

Systems like tilemaps and collision detection push users towards specific game mechanics and types
of game and limit their creativity.

However, these features are still essential for a lot of games.

Thus, we provide both low level features like drawing an image at a position and higher level features like an entity system.

In documentation, we try to first present the low level feature and build toward higher level concept to not influence the user.

## Interface with most tools

We try to support common file formats including (but not limited to) `gltf` and `fbx` for 3d objects, `png`, `gif` and `jpg` for images,
`wav`, `mp3` and `flac` for audio, `tmx` for levels, etc.

We cannot create the best code editor, 3d modeling software or level editor, but we can seamlessly integrate with the best ones.

With hot reload, Vectarine needs to feel like an extension of Blender, VSCode, Aseprite, Tiled, LDtk or FLStudio.

Because these tools are not integrated into the editor, we need to guide the user to them through links.

# Run anywhere without lag

## Export to anything

A vectarine game that should run identically in the editor on one platform as
when exported on another platform.

For example, we force paths to be case sensitive and use `/` to avoid games working by
accident on case insensitive systems, but breaking when exported.

## Be performant, even on low-end devices

Just like other interpreted languages like Python, Lua loops are slower than their native counterparts.

We should provide APIs to allow the user to move their computation heavy code to the native platform.

Performance critial parts of the native code need to be written carefully to take advantage of CPU features like SIMD
and while being aware of data locality.

# Provide structure to code

Most people that want to create games do not have a programing background.
We need to offer them solutions to naturally organize their projects so
that they are understandable to them and other.

Lua is not object-oriented by default, but it can be.

# Making buggy code harder to write than correct code

We need to create APIs that are naturally hard to misuse.

One example is the `Canvas:paint` which takes a function that draws to the canvas.
This prevents the user from forgetting that they are drawing to the canvas as everything is wrapped in an indented block.
It also means that the user cannot forget to stop drawing to the canvas.
