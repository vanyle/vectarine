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
Runs the CLI of the latest GitHub release, with the editor and the CLI replaced by debug builds of the current code.

Usage: uv run scripts/run-bundled-cli.py export --project path/to/game.vecta --target web
Options before the CLI arguments:
--export-folder path/to/folder: extract the exported zip into that folder
--debug-web: replace the web runtime of the release by a debug build of the current code
--debug-native: same for the runtime of the current platform
--serve: serve the export folder on http://localhost:8000
"""

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import urllib.request
import zipfile
from pathlib import Path

from rich.console import Console  # type: ignore

console = Console()

RELEASE_API_URL = "https://api.github.com/repos/vanyle/vectarine/releases/latest"

root_path = Path(__file__).parent.parent
cache_path = root_path / "target" / "latest-release"
bundle_path = cache_path / "bundle"
release_runtimes_path = cache_path / "release-runtimes"
web_files = ["runtime.js", "runtime.wasm", "index.html"]
exe = ".exe" if os.name == "nt" else ""

match platform.system():
    case "Darwin":
        release_name = "vectarine.macos.arm64"
        editor_in_bundle = "VectarineEditor.app/VectarineEditor"
        native_runtime = "runtime-macos"
    case "Windows":
        release_name = "vectarine.windows.x86_64"
        editor_in_bundle = "VectarineEditor.exe"
        native_runtime = "runtime.exe"
    case _:
        release_name = "vectarine.linux.x86_64"
        editor_in_bundle = "VectarineEditor"
        native_runtime = "runtime-linux"
cli_in_bundle = f"vecta{exe}"


def get_latest_release_url() -> tuple[str, str] | None:
    try:
        with urllib.request.urlopen(RELEASE_API_URL) as response:
            release = json.load(response)
    except Exception as e:
        console.print(f"[yellow]Could not get the latest release from GitHub: {e}")
        return None
    for asset in release["assets"]:
        if asset["name"] == f"{release_name}.zip":
            return release["tag_name"], asset["browser_download_url"]
    console.print(f"[yellow]The latest release has no {release_name}.zip")
    return None


def extract_with_permissions(zip_path: Path, dest: Path) -> None:
    with zipfile.ZipFile(zip_path) as archive:
        for info in archive.infolist():
            extracted = archive.extract(info, dest)
            mode = info.external_attr >> 16
            if mode and os.name != "nt":
                os.chmod(extracted, mode)


def update_release() -> None:
    tag_file = cache_path / "tag.txt"
    cached_tag = tag_file.read_text().strip() if tag_file.exists() else None
    latest = get_latest_release_url()
    if latest is None:
        if cached_tag is None:
            console.print("[red]No release in the cache, and the latest one could not be downloaded")
            sys.exit(1)
        console.print(f"[yellow]Using the cached release {cached_tag}")
        return

    tag, url = latest
    if tag == cached_tag and bundle_path.exists() and release_runtimes_path.exists():
        return

    console.print(f"[blue]Downloading release {tag} from {url}")
    cache_path.mkdir(parents=True, exist_ok=True)
    zip_path = cache_path / f"{release_name}.zip"
    with urllib.request.urlopen(url) as response, open(zip_path, "wb") as f:
        shutil.copyfileobj(response, f)

    shutil.rmtree(bundle_path, ignore_errors=True)
    extract_with_permissions(zip_path, bundle_path)
    zip_path.unlink()
    shutil.rmtree(release_runtimes_path, ignore_errors=True)
    release_runtimes_path.mkdir()
    for name in [*web_files, native_runtime]:
        shutil.copy2(bundle_path / name, release_runtimes_path / name)
    tag_file.write_text(tag)


def build_and_swap_executables() -> None:
    for package in ["editor", "vectarine-cli"]:
        console.print(f"[blue]Building {package}")
        subprocess.run(["cargo", "build", "-p", package], cwd=root_path, check=True)

    built_path = root_path / "target" / "debug"
    for built, in_bundle in [(f"vecta{exe}", editor_in_bundle), (f"vectarine-cli{exe}", cli_in_bundle)]:
        (bundle_path / in_bundle).unlink(missing_ok=True)
        shutil.copy2(built_path / built, bundle_path / in_bundle)


def swap_runtimes(debug_web: bool, debug_native: bool) -> None:
    if debug_web:
        console.print("[blue]Building the web runtime")
        subprocess.run(["cargo", "build", "-p", "runtime", "--target", "wasm32-unknown-emscripten"], cwd=root_path, check=True)
        built_path = root_path / "target" / "wasm32-unknown-emscripten" / "debug"
        sources = {name: built_path / name for name in ["runtime.js", "runtime.wasm"]}
        sources["index.html"] = root_path / "index.html"
    else:
        sources = {name: release_runtimes_path / name for name in web_files}

    if debug_native:
        console.print("[blue]Building the runtime")
        subprocess.run(["cargo", "build", "-p", "runtime"], cwd=root_path, check=True)
        sources[native_runtime] = root_path / "target" / "debug" / f"runtime{exe}"
    else:
        sources[native_runtime] = release_runtimes_path / native_runtime

    for folder in [bundle_path, bundle_path / "VectarineEditor.app"]:
        if folder.exists():
            for name, source in sources.items():
                (folder / name).unlink(missing_ok=True)
                shutil.copy2(source, folder / name)


def main() -> None:
    parser = argparse.ArgumentParser(allow_abbrev=False)
    parser.add_argument("--export-folder", type=Path)
    parser.add_argument("--debug-web", action="store_true")
    parser.add_argument("--debug-native", action="store_true")
    parser.add_argument("--serve", action="store_true")
    options, cli_args = parser.parse_known_args()
    export_folder = options.export_folder
    if export_folder is not None:
        export_zip = export_folder.with_name(f"{export_folder.name}.zip")
        export_folder.parent.mkdir(parents=True, exist_ok=True)
        cli_args = [*cli_args, "--output", str(export_zip)]

    update_release()
    build_and_swap_executables()
    swap_runtimes(options.debug_web, options.debug_native)
    cli_path = bundle_path / cli_in_bundle
    console.print(f"[blue]Running {cli_path} {' '.join(cli_args)}")
    returncode = subprocess.run([str(cli_path), *cli_args]).returncode

    if export_folder is not None and returncode == 0:
        shutil.rmtree(export_folder, ignore_errors=True)
        extract_with_permissions(export_zip, export_folder)
        export_zip.unlink()
        console.print(f"[green]Exported to {export_folder}")
        if options.serve:
            subprocess.run([sys.executable, str(root_path / "scripts" / "serve.py"), "8000", str(export_folder)])
    sys.exit(returncode)


if __name__ == "__main__":
    main()
