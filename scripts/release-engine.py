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

from rich.console import Console  # type: ignore

is_windows = os.name == "nt"


def copy_from_root(root_path: str, src: str, dst: str) -> None:
    src_file = os.path.join(root_path, src)
    if os.path.exists(src_file):
        shutil.copyfile(src_file, os.path.join(root_path, dst))


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
    release_path = os.path.join(root_path, "engine-release")
    shutil.rmtree(release_path, ignore_errors=True)
    Path(release_path).mkdir(parents=True, exist_ok=True)

    if is_windows:
        copy_from_root(
            root_path, "target/release/vecta.exe", "engine-release/vecta.exe"
        )
        copy_from_root(
            root_path, "target/release/runtime.exe", "engine-release/runtime.exe"
        )
        # If there is a linux release in the target, we ship it too.
        copy_from_root(
            root_path,
            "target/x86_64-unknown-linux-gnu/release/vecta",
            "engine-release/vecta-linux",
        )
        copy_from_root(
            root_path,
            "target/x86_64-unknown-linux-gnu/release/runtime",
            "engine-release/runtime-linux",
        )
    else:
        copy_from_root(root_path, "target/release/vecta", "engine-release/vecta")
        copy_from_root(root_path, "target/release/runtime", "engine-release/runtime")

    copy_from_root(
        root_path, "docs/user_manual.md", "engine-release/vectarine_guide.md"
    )

    copy_from_root(
        root_path,
        "target/wasm32-unknown-emscripten/release/runtime.js",
        "engine-release/runtime.js",
    )

    copy_from_root(
        root_path,
        "target/wasm32-unknown-emscripten/release/runtime.wasm",
        "engine-release/runtime.wasm",
    )
    copy_from_root(root_path, "index.html", "engine-release/index.html")

    shutil.copytree(
        os.path.join(root_path, "lua-api"),
        os.path.join(root_path, "engine-release/lua-api"),
        dirs_exist_ok=True,
    )
    shutil.copytree(
        os.path.join(root_path, "assets"),
        os.path.join(root_path, "engine-release/assets"),
    )

    console.print("[blue]Patching")
    index_html = ""
    with open(os.path.join(root_path, "engine-release/index.html"), "r") as f:
        index_html = f.read()
        index_html = index_html.replace(
            "target/wasm32-unknown-emscripten/release/runtime.js",
            "runtime.js",
        ).replace(
            "target/wasm32-unknown-emscripten/debug/runtime.js",
            "runtime.js",
        )
    with open(os.path.join(root_path, "engine-release/index.html"), "w") as f:
        f.write(index_html)

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
