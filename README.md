<p align="center">
    <img src="./assets/logo.png" alt="Vectarine logo" width="200" align="center"/>
</p>

<h1 align="center">Vectarine</h1>

<p align="center">
<a href="https://github.com/vanyle/vectarine/actions/workflows/test.yml"><img src="https://github.com/vanyle/vectarine/actions/workflows/test.yml/badge.svg" alt="Build & Test CI status badge"></a>
<a href="LICENSE"><img src="https://img.shields.io/github/license/vanyle/vectarine.svg" alt="License badge"></a>
<a href="https://discord.gg/zPwg3VDydz"><img src="https://dcbadge.limes.pink/api/server/zPwg3VDydz?style=flat" alt="Discord invite badge" /></a>
</p>

*[Vectarine](https://vectarine.surge.sh) is a game engine with a focus on ultra fast prototyping, ease of use and having fun.*

## Goals by importance

- Your time is valuable
  - **Luau** scripting: Instant reload and strong typing
  - Assets built into the engine for fast testing
  - Gallery of example: start with working templates
  - Powerful debugging tools and editor: waste less time on bugs, boilerplate and clicking around in menus
- Don't limit creativity
  - Access to low-level primitives
  - 3d & 2d support
  - Performance: Render millions of entities at 60 fps
  - Extensible: Write and share **Rust plugins** that can add anything to the engine
- Reach a wide audience
  - Supports the Web, Windows, Linux, MacOS
  - Distribute your game by sharing a zip with a small size footprint.
  - Free and open-source

## Getting started making games

Download [the latest build](https://github.com/vanyle/vectarine/releases/latest) of the engine from the [releases](https://github.com/vanyle/vectarine/releases) page.

Additionally, you'll need:

- A text editor. I recommend [Visual Studio Code](https://code.visualstudio.com/), but you can use Notepad or anything really.
- (Optional) Installing a Luau extension for your editor

> [!TIP]
> You can also install and update Vectarine to the latest version using these one-liners:
>
>
> On Linux: `curl -fsSL https://vectarine.surge.sh/install_linux.sh | sh`
>
> On Mac: `curl -fsSL https://vectarine.surge.sh/install_mac.sh | sh`
>
> On Windows with Powershell: `irm https://vectarine.surge.sh/install_nt.sh | iex`
>

<br>

See **[The guided tour](https://vectarine.surge.sh/guides/a-guided-tour.html)** for detailed information on how to make games with vectarine.

If this is your first time making games, read the guide to [create your first game](https://vectarine.surge.sh/guides/getting-started.html)

If you prefer watching rather than reading, there is a **[video presentation](https://www.youtube.com/watch?v=KwckT9mbj10)** to get started.

Feel free to join our [Discord server](https://discord.gg/zPwg3VDydz) if you have any questions or want to chat with other developers.

Below are information on how to improve the engine.

## Plugins

Vectarine can be extended with [**plugins** written in **Rust**](https://github.com/vanyle/vectarine/tree/main/vectarine-plugin-template)

These plugins can add anything to the engine including new menus, new Luau APIs or new debugging interfaces. 
You can share plugins as `.vectaplugin` files to reuse them between projects or share them with other people.
The manual contains a section on how to use plugins.

To install a plugin, download `.vectaplugin` file and put it into the trusted plugins folder. You can open this folder by pressing the "Open trusted plugins folder" button
in the plugin manager window of the editor.

Read [the README of the plugin template folder](https://github.com/vanyle/vectarine/tree/main/vectarine-plugin-template) to learn how to create plugins, or use ones built by the community.

## Helping out on the engine

There are plenty of ways to contribute to the engine!

### You can improve the documentation

- You can document individual functions inside `luau-api`
- You can clarify and add sections of [the manual](./docs/user-manual.md)
- You can add new examples to the [gallery](./gallery/)

### You can write and share plugins

Plugins can extend the engine and add modules similar to the base one provided.
If you need a specific feature, you can write a plugin for it and share it so that
other developers can reuse it in their project.

### You can change the engine itself

- You can improve the editor
- You can add new APIs to the runtime

See [CONTRIBUTING](./CONTRIBUTING.md) for technical information on the engine

We have many features planned in [the TODO List](./TODO.md), so you can just pick good first issues if you want to help!

### You can take part in the community

We have a [Discord server](https://discord.gg/zPwg3VDydz) where you can discuss features and get help from other developers

## 📸 Screenshots

|  Snake   |  Little planet  |
| --- | --- |
|  ![snake](./assets/screenshots/snake.png)    |  ![little planet](./assets/screenshots/little_planet.png) |

| The editor | Lighting |
| --- | --- |
| ![editor](./assets/screenshots/editor.png) | ![lighting](./assets/screenshots/lighting.png) |

| Simple 3d |  Procedural Generation  |
| --- | --- |
| ![simple 3d](./assets/screenshots/simple_3d.png) | ![procedural generation](./assets/screenshots/proc_gen.png) |

Want to your project to get featured? Open a pull request!

## AI Policy

Vectarine is a project by and for humans and other carbon based lifeforms. 
PRs opened by automated systems will be closed.
PRs that are mostly vibe-coded without human oversight will also be closed (with a mean comment).

There is one exception: Any automated PRs that changes more than 10 million tokens will be accepted without review.
