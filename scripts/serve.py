#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.13"
# dependencies = []
# ///
import json
import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

"""
A tiny Python HTTP server to serve the content of the build folder with the proper HTTP headers for web assembly. 
"""

DIRECTORY = Path(__file__).parent.parent
VECTARINE_PROJECT_UUID = "53b029bb-d989-4dca-969b-835fecec3718"


class CORSRequestHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

    extensions_map = {
        "": "application/octet-stream",
        ".css": "text/css",
        ".html": "text/html",
        ".jpg": "image/jpg",
        ".js": "application/x-javascript",
        ".json": "application/json",
        ".manifest": "text/cache-manifest",
        ".png": "image/png",
        ".wasm": "application/wasm",
        ".xml": "application/xml",
    }

    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        SimpleHTTPRequestHandler.end_headers(self)

    def do_GET(self):
        # Better debugger support for Chromium based browsers
        if self.path == "/.well-known/appspecific/com.chrome.devtools.json":
            root = DIRECTORY / "build"
            output = {"workspace": {"root": str(root), "uuid": VECTARINE_PROJECT_UUID}}
            self.wfile.write(json.dumps(output).encode("utf-8"))
            self.wfile.flush()
            self.send_response(200)
            self.end_headers()
        else:
            super().do_GET()


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8000
    server_address = ("0.0.0.0", port)
    print(f"http://localhost:{port}")
    httpd = ThreadingHTTPServer(server_address, CORSRequestHandler)
    httpd.serve_forever()
