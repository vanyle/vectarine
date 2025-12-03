# Lua code organisation

At first, `game.lua` is loaded.
We need a way to allow spliting the code into modules while still keeping everything hotreload friendly.

Also, the way we do things needs to be friendly to the IDE and the Lua LSP.

```lua
exported_function = require(filename.lua)
```

We will provide the following organisational primitives:

### Resources

A resource is an external file with a name and some dependencies.
We do not allow dependency cycles for resources.

### Store

A store is a global piece of state.
Store uses keys which are strings. Values can be arbitrary data.
There can be metadata associated with the keys:

- Is the key persisted (saved?)
- You can subscribe to changes of the state.

Global state is super important in games (and in all apps), so it makes sense to have it as a global primitive.
Inlike regular global state, our global state will be super debuggable as we are aware at run-time of what is going on.

### Bus

There is a bus where people can post events and subscribe to events.
Events have a name + data. In the debugger, you'll be able to filter events (per name, per emiting module or per receiving module)

In the future, the bus could be extended for multiplayer?

## What Love2d does

Just require() calls

## What defold does

Defold uses modules which talk to each other through message passing.
This is nice, but it means that exchanges messages are not typed.
The upside is that the hot reloading is super elegant as the modules can be loaded and unloaded individually.

Player character = one module
Obstacle = one module
In general, there is one module per entity type.
