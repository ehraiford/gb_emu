#!/usr/bin/env python3
"""Dumps the BOOTROM array from bootrom.rs to a flat binary.

The reference core has to boot from the same 256 bytes gb_emu does, or the two traces diverge
before the cartridge ever starts running.
"""
import re
import sys

source, dest = sys.argv[1], sys.argv[2]
body = open(source, encoding="utf-8").read().split("const BOOTROM: [u8; 256] = [", 1)[1].split("];", 1)[0]
values = [int(a, 16) if a else int(b) for a, b in re.findall(r"0x([0-9A-Fa-f]{2})|(?<![\w.])(\d{1,3})(?![\w.])", body)]
if len(values) != 256:
    raise SystemExit(f"expected 256 boot ROM bytes, parsed {len(values)}")
open(dest, "wb").write(bytes(values))
print(f"wrote {dest}")
