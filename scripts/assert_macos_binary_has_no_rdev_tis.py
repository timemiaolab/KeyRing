#!/usr/bin/env python3
"""Assert a macOS KeyRing binary never links rdev character translation.

rdev 0.5.3's Keyboard::string_from_code calls TIS/HIToolbox APIs that abort
with _dispatch_assert_queue_fail when invoked from a background event-tap
thread. macOS builds must not contain those symbols.
"""
from __future__ import annotations

import sys
from pathlib import Path


def needles() -> list[bytes]:
    # Split so this script's own source is not a false positive if someone
    # concatenates it into a blob, and so a Rust test can construct the same
    # strings without embedding them as contiguous literals.
    return [
        b"string_from" + b"_code",
        b"TISCopyCurrentKeyboard" + b"InputSource",
        b"TISGetInputSource" + b"Property",
        b"TSMGetInputSource" + b"Property",
        b"create_string_for" + b"_key",
        # Rust crate-name mangling for `rdev` (`Nt..._4rdev`).
        b"4rd" + b"ev",
    ]


def check(path: Path) -> list[str]:
    data = path.read_bytes()
    hits = []
    for needle in needles():
        if needle in data:
            hits.append(needle.decode("ascii", "replace"))
    return hits


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: assert_macos_binary_has_no_rdev_tis.py <binary> [...]", file=sys.stderr)
        return 2
    failed = False
    for raw in sys.argv[1:]:
        path = Path(raw)
        if not path.is_file():
            print(f"missing binary: {path}", file=sys.stderr)
            failed = True
            continue
        hits = check(path)
        if hits:
            failed = True
            print(f"FORBIDDEN symbols in {path}: {', '.join(hits)}", file=sys.stderr)
        else:
            print(f"ok: {path} has no rdev/TIS character-translation symbols")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
