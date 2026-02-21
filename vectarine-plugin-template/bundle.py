#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.13"
# dependencies = [
#     "rich",
#     "toml",
# ]
# [tool.uv]
# exclude-newer = "2025-08-01T00:00:00Z"
# ///
"""
This script needs a `uv` installation to work.

bundle.py is a script used to package, merge and validate vectarine plugins.
Use `uv run bundle.py -h` for more information.
"""

import argparse
import os
import shutil
import subprocess
import sys
import zipfile

import toml
from rich.console import Console  # type: ignore

is_windows = os.name == "nt"

console = Console()

# Target definitions

# Each entry: (rust_target_triple, folder_name_in_zip, lib_filename)
# The lib_filename uses the placeholder {lib} which will be replaced by the
# actual Cargo lib name at runtime.
NATIVE_TARGETS = [
    ("x86_64-pc-windows-msvc", "windows", "{lib}.dll"),
    ("x86_64-unknown-linux-gnu", "linux", "lib{lib}.so"),
    ("aarch64-apple-darwin", "macos", "lib{lib}.dylib"),
]

# Emscripten WASM target
WASM_TARGET = ("wasm32-unknown-emscripten", "web", "{lib}.wasm")


def run(cmd: list[str], **kwargs) -> subprocess.CompletedProcess:
    """Run a command and return the result."""
    return subprocess.run(cmd, **kwargs)


def rustup_installed_targets() -> set[str]:
    """Return the set of targets currently installed in rustup."""
    result = run(["rustup", "target", "list", "--installed"], capture_output=True, text=True)
    if result.returncode != 0:
        return set()
    return {line.strip() for line in result.stdout.splitlines() if line.strip()}


def emcc_available() -> bool:
    """Return True if the emcc compiler is on the PATH."""
    return shutil.which("emcc") is not None


def cargo_lib_name(cargo_toml_path: str) -> str:
    """
    Read the Cargo.toml and return the library name.
    Cargo converts hyphens to underscores for the actual artifact filename.
    """
    with open(cargo_toml_path, "r") as f:
        data = toml.load(f)
    # The [lib] section may have an explicit name; otherwise fall back to [package].name
    lib_name = data.get("lib", {}).get("name") or data.get("package", {}).get("name", "plugin")
    return lib_name.replace("-", "_")


def build_target(triple: str, lib: str, release: bool) -> bool:
    """
    Run `cargo build [--release] --target <triple>`.
    Returns True on success, False on failure.
    """
    profile = "release" if release else "debug"
    console.print(f"  [cyan]Building[/cyan] [bold]{triple}[/bold] ({profile}) …")
    target_dir = os.path.join(os.getcwd(), "target")
    cmd = ["cargo", "build", "--target", triple, "--target-dir", target_dir]
    if release:
        cmd.append("--release")
    result = run(cmd, capture_output=False)
    return result.returncode == 0


def artifact_path(triple: str, lib_filename: str, release: bool) -> str | None:
    """
    Return the path to the compiled artifact, or None if it does not exist.
    Cargo outputs to target/<triple>/release/ or target/<triple>/debug/.
    """
    profile_dir = "release" if release else "debug"
    path = os.path.join("target", triple, profile_dir, lib_filename)
    if os.path.exists(path):
        return path
    return None


def main(release: bool = False):
    # 1. Check that the manifest exist and is valid
    try:
        with open("manifest.toml", "r") as f:
            manifest = toml.load(f)
        for field in ("name", "version", "description", "url"):
            if not manifest.get(field):
                console.print(f"[red]Error: manifest.toml is missing a '{field}' field[/red]")
                return
    except FileNotFoundError:
        console.print("[red]Error: manifest.toml not found[/red]")
        return
    except toml.TomlDecodeError as exc:
        console.print(f"[red]Error: manifest.toml is not valid TOML: {exc}[/red]")
        return

    plugin_name: str = str(manifest["name"]).strip()
    safe_name = plugin_name.replace(" ", "_").lower()

    console.print(f"[bold green]Bundling plugin:[/bold green] {plugin_name} - version: {manifest['version']}")

    # 2. Check that plugin.luau exists
    has_luau = os.path.exists("plugin.luau")
    if not has_luau:
        console.print(
            "[yellow]Warning:[/yellow] plugin.luau not found. Your plugin will have poor autocompletion support and lacks documentation."
        )

    # 3. Resolve the Cargo library name
    if not os.path.exists("Cargo.toml"):
        console.print("[red]Error: Cargo.toml not found. This is not a valid rust plugin for vectarine.[/red]")
        return

    lib = cargo_lib_name("Cargo.toml")

    # 4. Detect available targets and build
    installed = rustup_installed_targets()
    profile_dir = "release" if release else "debug"

    built: list[tuple[str, str, str]] = []  # (folder_in_zip, artifact_path, zip_entry_name)
    skipped: list[tuple[str, str]] = []  # (triple, reason)

    # Native targets
    for triple, folder, filename_tmpl in NATIVE_TARGETS:
        filename = filename_tmpl.format(lib=lib)
        if triple not in installed:
            skipped.append((triple, "target not installed (run: rustup target add " + triple + ")"))
            continue
        # We can only cross-compile to the current host or if the proper linker is set up,
        # but we attempt it regardless and report failure.
        ok = build_target(triple, lib, release)
        if not ok:
            skipped.append((triple, "build failed"))
            continue
        path = artifact_path(triple, filename, release)
        if path is None:
            skipped.append((triple, f"artifact not found at target/{triple}/{profile_dir}/{filename}"))
            continue
        built.append((folder, path, filename))

    # Emscripten / web target
    wasm_triple, wasm_folder, wasm_filename_tmpl = WASM_TARGET
    wasm_filename = wasm_filename_tmpl.format(lib=lib)
    if wasm_triple in installed and emcc_available():
        ok = build_target(wasm_triple, lib, release)
        if ok:
            path = artifact_path(wasm_triple, wasm_filename, release)
            if path is not None:
                built.append((wasm_folder, path, wasm_filename))
            else:
                skipped.append((wasm_triple, f"Artifact not found at target/{wasm_triple}/{profile_dir}/{wasm_filename}"))
        else:
            skipped.append((wasm_triple, "Build failed"))
    else:
        reasons = []
        if wasm_triple not in installed:
            reasons.append("wasm32-unknown-emscripten not installed")
        if not emcc_available():
            reasons.append("emcc not found on PATH")
        skipped.append((wasm_triple, "; ".join(reasons)))

    # 5. Report build summary
    console.print()
    console.rule("[bold]Build summary[/bold]")

    if built:
        console.print("[bold green]Built targets:[/bold green]")
        for folder, path, _ in built:
            console.print(f"  [green]✔[/green]  {folder:10s}  ({path})")
    else:
        console.print("[bold red]No targets were built successfully.[/bold red]")

    if skipped:
        console.print("[bold yellow]Skipped targets:[/bold yellow]")
        for triple, reason in skipped:
            console.print(f"  [yellow]✗[/yellow]  {triple:45s}  {reason}")

    if not built:
        console.print("\n[red]Nothing to bundle. Aborting.[/red]")
        sys.exit(1)

    # 6. Create the .vectaplugin zip
    output_filename = f"{safe_name}.vectaplugin"

    with zipfile.ZipFile(output_filename, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        # manifest.toml at root
        zf.write("manifest.toml", "manifest.toml")

        # plugin.luau at root (optional)
        if has_luau:
            zf.write("plugin.luau", "plugin.luau")

        # Native / wasm artifacts in their respective sub-folders.
        # Always named "plugin.<ext>" inside the zip (e.g. windows/plugin.dll)
        # regardless of the actual Cargo artifact name on disk.
        for folder, src_path, filename in built:
            ext = os.path.splitext(filename)[1]  # .dll / .so / .dylib / .wasm / other in the future?
            zf.write(src_path, f"{folder}/plugin{ext}")

    console.print()
    console.print(f"[bold green]✔ Plugin bundled:[/bold green] [cyan]{output_filename}[/cyan]")


KNOWN_FOLDERS = {"windows", "linux", "macos", "web"}
REQUIRED_MANIFEST_FIELDS = ("name", "version", "description", "url")


def validate_vectaplugin(path: str) -> tuple[dict, list[str]] | None:
    """
    Open *path* as a zip, validate its manifest.toml, and return
    ``(manifest_dict, list_of_zip_entry_names)`` on success or
    print an error and return ``None`` on failure.
    """
    if not os.path.exists(path):
        console.print(f"[red]Error:[/red] File not found: {path}")
        return None

    try:
        zf = zipfile.ZipFile(path, "r")
    except zipfile.BadZipFile:
        console.print(f"[red]Error:[/red] Not a valid zip file: {path}")
        return None

    with zf:
        names = zf.namelist()

        # Must contain a manifest.toml
        if "manifest.toml" not in names:
            console.print(f"[red]Error:[/red] {path} does not contain a manifest.toml")
            return None

        try:
            raw = zf.read("manifest.toml")
            manifest = toml.loads(raw.decode("utf-8"))
        except toml.TomlDecodeError as exc:
            console.print(f"[red]Error:[/red] {path}: manifest.toml is not valid TOML: {exc}")
            return None

        for field in REQUIRED_MANIFEST_FIELDS:
            if not manifest.get(field):
                console.print(f"[red]Error:[/red] {path}: manifest.toml is missing the '{field}' field")
                return None

        # Must contain at least one known platform folder with an actual file in it
        has_platform = any(name.split("/")[0] in KNOWN_FOLDERS and name.split("/")[-1] != "" for name in names if "/" in name)
        if not has_platform:
            console.print(
                f"[red]Error:[/red] {path} contains no recognised platform folders with files ({', '.join(sorted(KNOWN_FOLDERS))})"
            )
            return None

        return manifest, names


def merge_plugins(input_paths: list[str]) -> None:
    """
    Merge several .vectaplugin files into one.
    The first file listed takes priority in case of conflicting entries.
    """
    console.rule("[bold]Merging the provided plugins...[/bold]")

    # Validate all inputs first so we fail fast before writing anything
    validated: list[tuple[str, dict, list[str]]] = []  # (path, manifest, names)
    for path in input_paths:
        result = validate_vectaplugin(path)
        if result is None:
            sys.exit(1)
        manifest, names = result
        validated.append((path, manifest, names))
        console.print(f"  [green]✔[/green]  [cyan]{path}[/cyan]  — {manifest['name']} v{manifest['version']}")

    # Check that all plugins share the same name (sanity guard)
    names_set = {v[1]["name"] for v in validated}
    if len(names_set) > 1:
        console.print("[yellow]Warning:[/yellow] merging plugins with different names: " + ", ".join(f"'{n}'" for n in sorted(names_set)))

    # Merge: first zip wins on conflict
    # We collect (arcname -> bytes) in insertion order; first occurrence wins.
    merged: dict[str, bytes] = {}
    sources: dict[str, str] = {}  # arcname -> originating file (for logging)

    for path, _manifest, _names in validated:
        with zipfile.ZipFile(path, "r") as zf:
            for info in zf.infolist():
                if info.filename not in merged:
                    merged[info.filename] = zf.read(info.filename)
                    sources[info.filename] = path
                else:
                    console.print(
                        f"  [dim]Conflict:[/dim] '{info.filename}' "
                        f"kept from [cyan]{sources[info.filename]}[/cyan], "
                        f"skipped from [dim]{path}[/dim]"
                    )

    first_manifest = validated[0][1]
    plugin_name = str(first_manifest["name"]).strip()
    safe_name = plugin_name.replace(" ", "_").lower()
    output_filename = f"{safe_name}.vectaplugin"

    with zipfile.ZipFile(output_filename, "w", compression=zipfile.ZIP_DEFLATED) as out_zf:
        for arcname, data in merged.items():
            out_zf.writestr(arcname, data)

    console.print()
    console.rule("[bold]Merge summary[/bold]")
    platform_entries = [arcname for arcname in merged if "/" in arcname and arcname.split("/")[0] in KNOWN_FOLDERS]
    for entry in sorted(platform_entries):
        console.print(f"  [green]✔[/green]  {entry}  [dim](from {sources[entry]})[/dim]")

    console.print()
    console.print(f"[bold green]✔ Merged plugin:[/bold green] [cyan]{output_filename}[/cyan]")


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="bundle.py",
        description="Build and package a Vectarine plugin (.vectaplugin).",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""\
Modes:
  (no arguments)          Build mode: compile the plugin for every installed
                          Rust target and package the result into a
                          <name>.vectaplugin zip file.

  <file.vectaplugin>      Validate mode: check that the given .vectaplugin is
                          well-formed and print a summary. Nothing is written.

  <a.vectaplugin> <b.vectaplugin> ...
                          Merge mode: validate every input file then merge
                          them into a single .vectaplugin. When two archives
                          contain the same entry, the first file wins.

Flags (build mode only):
  --release               Build with the release profile (optimised). When
                          omitted the debug profile is used instead.""",
    )
    parser.add_argument(
        "plugins",
        nargs="*",
        metavar="file.vectaplugin",
        help=".vectaplugin file(s) to validate or merge.",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        default=False,
        help="(build mode) compile with --release instead of the default debug profile.",
    )
    return parser


if __name__ == "__main__":
    parser = make_parser()
    opts = parser.parse_args()

    if len(opts.plugins) == 1:
        result = validate_vectaplugin(opts.plugins[0])
        if result is None:
            sys.exit(1)
        manifest, names = result
        platform_entries = [name for name in names if "/" in name and name.split("/")[0] in KNOWN_FOLDERS]
        console.print(f"[bold green]✔ Valid plugin:[/bold green] [cyan]{opts.plugins[0]}[/cyan]")
        console.print(f"  Name:     {manifest['name']}")
        console.print(f"  Version:  {manifest['version']}")
        console.print(f"  Targets:  {', '.join(sorted({n.split('/')[0] for n in platform_entries}))}")
    elif len(opts.plugins) > 1:
        merge_plugins(opts.plugins)
    else:
        main(release=opts.release)
