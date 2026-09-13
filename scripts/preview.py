#!/usr/bin/env python3
"""Local UI fixture only. Never included in the module or connected to a daemon."""

from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "webui/dist"
BRIDGE = Path(__file__).with_name("preview-fixture.js").read_bytes()


class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT), **kwargs)

    def do_GET(self):
        if self.path == "/fixture.js":
            content, mime = BRIDGE, "text/javascript"
        elif self.path.split("?", 1)[0] in ("/", "/index.html"):
            content = (
                (ROOT / "index.html")
                .read_bytes()
                .replace(b"</head>", b'<script src="./fixture.js"></script></head>')
            )
            mime = "text/html; charset=utf-8"
        else:
            return super().do_GET()
        self.send_response(200)
        self.send_header("Content-Type", mime)
        self.send_header("Content-Length", str(len(content)))
        self.end_headers()
        self.wfile.write(content)

    def log_message(self, *args):
        pass


print("WebUI fixture: http://127.0.0.1:4173 (mock data only)", flush=True)
ThreadingHTTPServer(("127.0.0.1", 4173), Handler).serve_forever()
