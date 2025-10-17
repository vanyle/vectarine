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
import sys
from pathlib import Path

from rich.console import Console  # type: ignore

os_friendly_name = "unknown"
platform = sys.platform

if platform.startswith("darwin"):
    os_friendly_name = "macos"
elif platform.startswith("linux"):
    os_friendly_name = "linux"
elif platform.startswith("win"):
    os_friendly_name = "windows"

is_windows = os.name == "nt"


def copy_from_root(root_path: str, src: str, dst: str, chmodx=False) -> None:
    src_file = os.path.join(root_path, src)
    if os.path.exists(src_file):
        shutil.copyfile(src_file, os.path.join(root_path, dst))
        if not is_windows and chmodx:
            st = os.stat(os.path.join(root_path, dst))
            os.chmod(os.path.join(root_path, dst), st.st_mode | 0o111)


def make_macos_app(root_path: str, executable_path: str, executable_dest_folder: str, friendly_name: str) -> None:
    """
    Example:
    ```
    make_macos_app(
        root_path,
        os.path.join(root_path, "target/aarch64-apple-darwin/release/vecta"),
        release_path,
        "Vectarine",
    )
    ```
    """
    if not os.path.exists(executable_path):
        return
    dest_executable_path = os.path.join(executable_dest_folder, f"{friendly_name}.app/Contents/MacOS/{friendly_name}")

    Path(os.path.join(executable_dest_folder, f"{friendly_name}.app/Contents/MacOS")).mkdir(parents=True, exist_ok=True)
    shutil.copyfile(
        executable_path,
        dest_executable_path,
    )
    if not is_windows:
        st = os.stat(dest_executable_path)
        os.chmod(dest_executable_path, st.st_mode | 0o111)

    # Write Info.plist
    with open(os.path.join(executable_dest_folder, f"{friendly_name}.app/Contents/Info.plist"), "w") as f:
        f.write(f"""
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>{friendly_name}</string>
  <key>CFBundleIdentifier</key>
  <string>com.vectarine.{friendly_name}</string>
</dict>
</plist>
""")


def main() -> None:
    root_path = str(Path(__file__).parent.parent)

    console = Console()
    console.print("[blue]Packaging the engine...")
    release_path = os.path.join(root_path, "engine-release")
    shutil.rmtree(release_path, ignore_errors=True)
    Path(release_path).mkdir(parents=True, exist_ok=True)

    if is_windows:
        copy_from_root(root_path, "target/release/vecta.exe", "engine-release/vecta.exe")
        copy_from_root(root_path, "target/release/runtime.exe", "engine-release/runtime.exe")
    else:
        copy_from_root(root_path, "target/release/vecta", f"engine-release/vecta-{os_friendly_name}", chmodx=True)
        copy_from_root(root_path, "target/release/runtime", f"engine-release/runtime-{os_friendly_name}", chmodx=True)

    # If there is a linux release in the target, we ship it too.
    copy_from_root(
        root_path,
        "target/x86_64-unknown-linux-gnu/release/vecta",
        "engine-release/vecta-linux",
        chmodx=True,
    )
    copy_from_root(
        root_path,
        "target/x86_64-unknown-linux-gnu/release/runtime",
        "engine-release/runtime-linux",
        chmodx=True,
    )
    # Same for macOS
    copy_from_root(
        root_path,
        "target/aarch64-apple-darwin/release/vecta",
        "engine-release/vecta-macos",
        chmodx=True,
    )
    copy_from_root(
        root_path,
        "target/aarch64-apple-darwin/release/vecta",
        "engine-release/runtime-macos",
        chmodx=True,
    )

    # Same for Windows
    copy_from_root(
        root_path,
        "target/x86_64-pc-windows-msvc/release/vecta.exe",
        "engine-release/vecta.exe",
    )
    copy_from_root(
        root_path,
        "target/x86_64-pc-windows-msvc/release/runtime.exe",
        "engine-release/runtime.exe",
    )

    copy_from_root(root_path, "docs/user_manual.md", "engine-release/vectarine_guide.md")

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
        os.path.join(root_path, "luau-api"),
        os.path.join(root_path, "engine-release/luau-api"),
        dirs_exist_ok=True,
    )
    shutil.copytree(
        os.path.join(root_path, "gallery"),
        os.path.join(root_path, "engine-release/gallery"),
    )
    shutil.copytree(
        os.path.join(root_path, "gamedata"),
        os.path.join(root_path, "engine-release/gamedata"),
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
