"""
This script is used to automate the testing of the Web version of Vectarine.

It:
- Builds the web version in debug mode
- Copies the generated files to the build folder with index.html and the gamedata
- Starts a local HTTP server to serve the content of the build folder

"""

import os
import subprocess
import sys
from pathlib import Path


def main():
    # Set the PWD to the project root to ensure cargo commands work correctly
    os.chdir(Path(__file__).parent.parent)

    # Build the web version in debug mode
    subprocess.run(["cargo", "build", "--target", "wasm32-unknown-emscripten", "-p", "runtime"], check=True)

    # Start the HTTP server
    subprocess.run([sys.executable, "scripts/serve.py"], check=True)


if __name__ == "__main__":
    main()
