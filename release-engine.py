#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "rich",
# ]
# [tool.uv]
# exclude-newer = "2025-08-01T00:00:00Z"
# ///

"""
Tool to generate an engine-release aka a distributable zip with the engine and resources compiled.
"""

import os
import shutil
import subprocess
from pathlib import Path

from rich.console import Console

is_windows = os.name == "nt"


def main() -> None:
    console = Console()
    console.print("[green]Building a release build of the engine.")
    console.print(
        "[green]Get a cup of coffee, tea or hot chocolate, this might take a while!"
    )

    console.print("[blue]Desktop build (runtime)")
    subprocess.run(
        ["cargo", "build", "-p", "runtime", "--release"],
        shell=True,
        stdout=subprocess.PIPE,
    )

    console.print("[blue]Desktop build (editor)")
    subprocess.run(
        ["cargo", "build", "-p", "editor", "--release"],
        shell=True,
        stdout=subprocess.PIPE,
    )

    # Note that the editor does not have a web build.
    console.print("[blue]Web build")
    subprocess.run(
        [
            "cargo",
            "build",
            "-p",
            "runtime",
            "--target",
            "wasm32-unknown-emscripten",
            "--release",
        ],
        shell=True,
        stdout=subprocess.PIPE,
    )

    console.print("[blue]Packaging")
    shutil.rmtree("engine-release", ignore_errors=True)
    Path("engine-release").mkdir(parents=True, exist_ok=True)

    if is_windows:
        shutil.copyfile("target/release/vecta.exe", "engine-release/vecta.exe")
        shutil.copyfile("target/release/runtime.exe", "engine-release/runtime.exe")
    else:
        shutil.copyfile("target/release/vecta", "engine-release/vecta")
        shutil.copyfile("target/release/runtime", "engine-release/runtime")

    shutil.copyfile(
        "target/wasm32-unknown-emscripten/release/runtime.js",
        "engine-release/runtime.js",
    )
    shutil.copyfile(
        "target/wasm32-unknown-emscripten/release/runtime.wasm",
        "engine-release/runtime.wasm",
    )
    shutil.copyfile(
        "index.html",
        "engine-release/index.html",
    )
    shutil.copytree(
        "lua-api",
        "engine-release/lua-api",
        dirs_exist_ok=True,
    )
    shutil.copyfile(
        "game.lua",
        "engine-release/game.lua",
    )
    console.print("[blue]Zipping")
    shutil.rmtree("vectarine.zip", ignore_errors=True)
    shutil.make_archive("vectarine", "zip", "engine-release")

    console.print("[green]Enjoy your vectarine!")


if __name__ == "__main__":
    main()
