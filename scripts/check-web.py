#!/usr/bin/env python3
"""Validate browser output, or smoke-test the same build on a static host."""
import argparse
import hashlib
from pathlib import Path
import re
import time
from urllib.request import Request, urlopen


def validate(read, expected=None):
    html = read("index.html").decode("utf-8")
    assert "__BUILD_VERSION__" not in html, "Unexpanded build version"
    versions = re.findall(r"\./airy(?:\.js|_bg\.wasm)\?v=([a-f0-9]{16})", html)
    assert len(versions) == 2 and versions[0] == versions[1], "JS/Wasm version mismatch"
    version = versions[0]
    assert not expected or version == expected, "Host is serving an older build"
    js = read(f"airy.js?v={version}")
    assert b"__wbg_init" in js, "Missing Wasm loader"
    wasm = read(f"airy_bg.wasm?v={version}")
    assert wasm.startswith(b"\x00asm\x01\x00\x00\x00"), "Invalid Wasm response"
    size = re.search(r"const wasmBytes = ([0-9]+);", html)
    assert size and int(size[1]) == len(wasm), "Download progress size does not match Wasm"
    # Match build-web.sh's two sha256sum lines, including filenames and newlines.
    sums = "".join(
        f"{hashlib.sha256(data).hexdigest()}  dist/{name}\n"
        for name, data in (("airy.js", js), ("airy_bg.wasm", wasm))
    )
    assert hashlib.sha256(sums.encode()).hexdigest()[:16] == version, "JS/Wasm content does not match build version"
    for name, data in (("HTML", html.encode()), ("JS", js), ("Wasm", wasm)):
        assert not re.search(rb"/(?:home|Users)/[^/\s\x00]+/", data), f"Personal filesystem path in {name}"
    print(f"Verified browser build {version}: HTML, JS, Wasm, and path privacy.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--directory", type=Path)
    source.add_argument("--url")
    parser.add_argument("--version")
    parser.add_argument("--attempts", type=int, default=1)
    args = parser.parse_args()
    if args.directory:
        validate(lambda path: (args.directory / path.split("?")[0]).read_bytes(), args.version)
        return

    def download(path):
        request = Request(args.url.rstrip("/") + "/" + path, headers={"Cache-Control": "no-cache"})
        with urlopen(request, timeout=90) as response:
            mime = response.headers.get_content_type()
            if path.startswith("airy_bg.wasm"):
                assert mime == "application/wasm", f"Wrong Wasm MIME type: {mime}"
            elif path.startswith("airy.js"):
                assert mime in ("text/javascript", "application/javascript"), f"Wrong JS MIME type: {mime}"
            return response.read()

    for attempt in range(args.attempts):
        try:
            validate(download, args.version)
            return
        except (AssertionError, OSError, UnicodeError) as error:
            if attempt + 1 == args.attempts:
                raise
            print(f"Waiting for deployment: {error}", flush=True)
            time.sleep(10)


if __name__ == "__main__":
    main()
