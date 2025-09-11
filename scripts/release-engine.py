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
    root_path = str(Path(__file__).parent.parent)

    console = Console()
    console.print("[green]Building a release build of the engine.")
    console.print(
        "[green]Get a cup of coffee, tea or hot chocolate, this might take a while!"
    )

    console.print("[blue]Desktop build (runtime)")
    subprocess.run(
        ["cargo", "build", "-p", "runtime", "--release"],
        shell=True,
        cwd=root_path,
        stdout=subprocess.PIPE,
    )

    console.print("[blue]Desktop build (editor)")
    subprocess.run(
        ["cargo", "build", "-p", "editor", "--release"],
        shell=True,
        cwd=root_path,
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
        cwd=root_path,
        stdout=subprocess.PIPE,
    )

    console.print("[blue]Packaging")
    shutil.rmtree("engine-release", ignore_errors=True)
    Path("engine-release").mkdir(parents=True, exist_ok=True)

    if is_windows:
        shutil.copyfile(
            os.path.join(root_path, "target/release/vecta.exe"),
            os.path.join(root_path, "engine-release/vecta.exe"),
        )
        shutil.copyfile(
            os.path.join(root_path, "target/release/runtime.exe"),
            os.path.join(root_path, "engine-release/runtime.exe"),
        )
    else:
        shutil.copyfile(
            os.path.join(root_path, "target/release/vecta"),
            os.path.join(root_path, "engine-release/vecta"),
        )
        shutil.copyfile(
            os.path.join(root_path, "target/release/runtime"),
            os.path.join(root_path, "engine-release/runtime"),
        )

    shutil.copyfile(
        os.path.join(root_path, "target/wasm32-unknown-emscripten/release/runtime.js"),
        os.path.join(root_path, "engine-release/runtime.js"),
    )
    shutil.copyfile(
        os.path.join(
            root_path, "target/wasm32-unknown-emscripten/release/runtime.wasm"
        ),
        os.path.join(root_path, "engine-release/runtime.wasm"),
    )
    shutil.copyfile(
        os.path.join(root_path, "index.html"),
        os.path.join(root_path, "engine-release/index.html"),
    )
    shutil.copytree(
        os.path.join(root_path, "lua-api"),
        os.path.join(root_path, "engine-release/lua-api"),
        dirs_exist_ok=True,
    )
    shutil.copyfile(
        os.path.join(root_path, "game.lua"),
        os.path.join(root_path, "engine-release/game.lua"),
    )
    console.print("[blue]Zipping")
    shutil.rmtree(os.path.join(root_path, "vectarine.zip"), ignore_errors=True)
    console.print(f"[blue]Creating {os.path.join(root_path, 'vectarine.zip')} ...")
    shutil.make_archive(
        base_name=os.path.join(root_path, "vectarine"),
        format="zip",
        root_dir=os.path.join(root_path, "engine-release"),
        # base_dir=root_path,
    )

    console.print("[green]Enjoy your vectarine!")


if __name__ == "__main__":
    main()
