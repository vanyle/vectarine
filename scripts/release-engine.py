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
Tool to generate an engine-release aka a distributable zips with the engine and resources compiled.
"""

import os
import shutil
from pathlib import Path

from rich.console import Console  # type: ignore

is_windows = os.name == "nt"

console = Console()


def copy_from_root(root_path: str, src: str, dst: str, chmodx=False) -> None:
    src_file = os.path.join(root_path, src)
    if os.path.exists(src_file):
        shutil.copyfile(src_file, os.path.join(root_path, dst))
        if not is_windows and chmodx:
            st = os.stat(os.path.join(root_path, dst))
            os.chmod(os.path.join(root_path, dst), st.st_mode | 0o111)
    else:
        console.print(f"[yellow]Could not find {src_file} to copy")
        console.print("[yellow]Your release will usable, but incomplete as runtimes will be missing")


def copy_runtime_files(root_path: str, dest: str = ""):
    if dest == "":
        dest = "engine-release"
    # Pack the runtimes for all platforms if possible
    copy_from_root(root_path, "target/x86_64-pc-windows-msvc/release/runtime.exe", os.path.join(dest, "runtime.exe"))
    copy_from_root(root_path, "target/x86_64-unknown-linux-musl/release/runtime", os.path.join(dest, "runtime-linux"), chmodx=True)
    copy_from_root(root_path, "target/aarch64-apple-darwin/release/runtime", os.path.join(dest, "runtime-macos"), chmodx=True)
    copy_from_root(root_path, "target/wasm32-unknown-emscripten/release/runtime.js", os.path.join(dest, "runtime.js"))
    copy_from_root(root_path, "target/wasm32-unknown-emscripten/release/runtime.wasm", os.path.join(dest, "runtime.wasm"))

    console.print("[blue]Patching index.html")
    index_html = ""
    with open(os.path.join(root_path, "index.html"), "r") as f:
        index_html = f.read()
        index_html = index_html.replace(
            "target/wasm32-unknown-emscripten/release/runtime.js",
            "runtime.js",
        ).replace(
            "target/wasm32-unknown-emscripten/debug/runtime.js",
            "runtime.js",
        )
    with open(os.path.join(root_path, os.path.join(dest, "index.html")), "w+") as f:
        f.write(index_html)


def copy_lua_and_docs(root_path: str):
    copy_from_root(root_path, "docs/user-manual.md", "engine-release/vectarine-guide.md")
    copy_from_root(root_path, "docs/user-manual.pdf", "engine-release/vectarine-guide.pdf")

    shutil.copytree(
        os.path.join(root_path, "luau-api"),
        os.path.join(root_path, "engine-release/luau-api"),
        dirs_exist_ok=True,
    )
    shutil.copytree(
        os.path.join(root_path, "gallery"),
        os.path.join(root_path, "engine-release/gallery"),
    )
    # We copy gamedata so that the runtime works by default, but we might not do so in the future.
    # gamedata is the folder loaded by default by the runtime when there is no bundle.vecta file available.
    shutil.copytree(
        os.path.join(root_path, "gamedata"),
        os.path.join(root_path, "engine-release/gamedata"),
    )


def get_clean_engine_release_folder(root_path: str) -> str:
    release_path = os.path.join(root_path, "engine-release")
    shutil.rmtree(release_path, ignore_errors=True)
    Path(release_path).mkdir(parents=True, exist_ok=True)
    return release_path


def make_windows_release(root_path: str) -> bool:
    output_zip_name = "vectarine.windows.x86_64"
    console.print("[blue]Trying to package the engine for Windows...")
    release_path = get_clean_engine_release_folder(root_path)

    executable_path = os.path.join(root_path, "target/x86_64-pc-windows-msvc/release/vecta.exe")
    if not os.path.exists(executable_path):
        console.print("[red]Could not find the editor executable to package the engine for Windows!")
        return False

    copy_from_root(root_path, "target/x86_64-pc-windows-msvc/release/vecta.exe", "engine-release/vecta.exe")
    copy_runtime_files(root_path)
    copy_lua_and_docs(root_path)

    shutil.rmtree(os.path.join(root_path, output_zip_name), ignore_errors=True)
    shutil.make_archive(
        base_name=os.path.join(root_path, output_zip_name),
        format="zip",
        root_dir=release_path,
    )

    console.print(f"[green]Successfully packaged the engine for Windows at {output_zip_name}!")
    return True


def make_linux_release(root_path: str) -> bool:
    output_zip_name = "vectarine.linux.x86_64"
    console.print("[blue]Trying to package the engine for Linux...")
    release_path = get_clean_engine_release_folder(root_path)

    executable_path = os.path.join(root_path, "target/x86_64-unknown-linux-musl/release/vecta")
    if not os.path.exists(executable_path):
        console.print("[red]Could not find the editor executable to package the engine for Linux!")
        return False

    copy_from_root(root_path, "target/x86_64-unknown-linux-musl/release/vecta", "engine-release/vecta", chmodx=True)
    copy_runtime_files(root_path)

    shutil.rmtree(os.path.join(root_path, output_zip_name), ignore_errors=True)
    shutil.make_archive(
        base_name=os.path.join(root_path, output_zip_name),
        format="zip",
        root_dir=release_path,
    )

    console.print("[green]Successfully packaged the engine for Linux!")
    return True


def make_macos_release(
    root_path: str,
) -> bool:
    # We generate an .app folder that we zip
    # Relevant docs:
    # https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html#//apple_ref/doc/uid/10000123i-CH101-SW1
    # https://developer.apple.com/documentation/bundleresources/information-property-list?language=objc
    output_zip_name = "vectarine.macos.arm64"  # no need to add .zip
    friendly_name = "vecta"
    console.print("[blue]Trying to package the engine for macOS...")
    release_path = get_clean_engine_release_folder(root_path)
    executable_path = os.path.join(root_path, "target/aarch64-apple-darwin/release/vecta")
    if not os.path.exists(executable_path):
        console.print("[red]Could not find the editor executable to package the engine for macOS!")
        return False
    executable_dest_folder = os.path.join(root_path, "engine-release")

    # The docs (including the gallery) needs to be outside for user discoverability.
    copy_lua_and_docs(root_path)

    # Put the gallery inside the bundle so that the start screen works
    shutil.copytree(
        os.path.join(root_path, "gallery"),
        os.path.join(root_path, f"engine-release/{friendly_name}.app/gallery"),
    )
    # For macOS, we put the runtime also in the bundle. Only the docs are outside
    copy_runtime_files(root_path, os.path.join(root_path, f"engine-release/{friendly_name}.app"))

    copy_from_root(root_path, "assets/logo.png", f"engine-release/{friendly_name}.app/vectaIcon.png")
    copy_from_root(root_path, "assets/logo.png", f"engine-release/{friendly_name}.app/Default.png")

    copy_from_root(root_path, "target/aarch64-apple-darwin/release/vecta", f"engine-release/{friendly_name}.app/vecta", chmodx=True)

    # Write Info.plist
    infoplist_path = f"{friendly_name}.app/Info.plist"
    Path(os.path.join(executable_dest_folder, infoplist_path)).parent.mkdir(parents=True, exist_ok=True)

    with open(
        os.path.join(executable_dest_folder, infoplist_path),
        "w+",
    ) as f:
        f.write(f"""
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>{friendly_name}</string>
  <key>CFBundleIdentifier</key>
  <string>com.vectarine.{friendly_name}</string>
  <key>CFBundleDisplayName</key>
  <string>{friendly_name}</string>
  <key>CFBundleIconFile</key>
  <string>Default.png</string>
  <key>CFBundleIconFiles</key>
  <array>
    <string>Default.png</string>
  </array>
</dict>
</plist>
""")
    shutil.rmtree(os.path.join(root_path, output_zip_name), ignore_errors=True)
    shutil.make_archive(
        base_name=os.path.join(root_path, output_zip_name),
        format="zip",
        root_dir=release_path,
    )

    console.print("[green]Successfully packaged the engine for macOS!")
    return True


def main() -> None:
    root_path = str(Path(__file__).parent.parent)

    console.print("[blue]Packaging the engine...")
    at_least_one_success = False
    at_least_one_success = make_windows_release(root_path) or at_least_one_success
    at_least_one_success = make_linux_release(root_path) or at_least_one_success
    at_least_one_success = make_macos_release(root_path) or at_least_one_success
    if not at_least_one_success:
        console.print("[red]Failed to package the engine, you need to compile the engine for at least one platform!")
        console.print("[red]This script only bundles files, it does not compile anything")
    else:
        console.print("[green]Enjoy your vectarine!")
    get_clean_engine_release_folder(root_path)


if __name__ == "__main__":
    main()
