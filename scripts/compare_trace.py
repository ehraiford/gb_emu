#!/usr/bin/env python3
"""Trace a ROM on both cores and report where gb_emu diverges from SameBoy.

  python scripts/compare_trace.py <rom> [--instructions=N] [--context=N] [--from-cycle=N]
                                       [--ignore=col,col] [--offset=N]

Build the reference first with tools/reference_tracer/build.sh. gb_emu is traced for more
M-cycles than the reference needs, since the reference's per-instruction samples are a sparse
subset of gb_emu's per-M-cycle ones.
"""
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TRACER = os.path.join(ROOT, "tools", "reference_tracer", "tracer.exe")
BOOTROM = os.path.join(ROOT, "tools", "reference_tracer", "dmg_boot.bin")
OUT_DIR = os.path.join(ROOT, "target", "traces")

# An instruction averages a little over two M-cycles; the slack keeps gb_emu's trace long enough
# to cover every reference sample even when it runs ahead.
CYCLES_PER_INSTRUCTION = 4


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    flags = {a.split("=")[0]: a.split("=")[1] for a in sys.argv[1:] if a.startswith("--") and "=" in a}
    if not args:
        print(__doc__)
        return 2
    rom = args[0]
    instructions = int(flags.get("--instructions", 200_000))
    # Passed to both tracers so neither writes the boot ROM's million-cycle logo scroll.
    skip = flags.get("--from-cycle", "0")

    if not os.path.exists(TRACER):
        print(f"reference tracer not built; run tools/reference_tracer/build.sh")
        return 1

    os.makedirs(OUT_DIR, exist_ok=True)
    name = os.path.splitext(os.path.basename(rom))[0]
    ref_trace = os.path.join(OUT_DIR, f"{name}.reference.trace")
    gb_trace = os.path.join(OUT_DIR, f"{name}.gb_emu.trace")

    print(f"tracing {name} on SameBoy ({instructions} instructions)...")
    subprocess.run([TRACER, rom, BOOTROM, str(instructions), ref_trace, skip], check=True)

    print(f"tracing {name} on gb_emu...")
    subprocess.run(
        ["cargo", "run", "--release", "--quiet", "--features", "headless", "--example", "trace",
         "--", rom, str(instructions * CYCLES_PER_INSTRUCTION), gb_trace, skip],
        check=True, cwd=ROOT,
    )

    print()
    diff = [sys.executable, os.path.join(ROOT, "scripts", "trace_diff.py"), ref_trace, gb_trace]
    for passthrough in ("--context", "--from-cycle", "--ignore", "--offset"):
        if passthrough in flags:
            diff.append(f"{passthrough}={flags[passthrough]}")
    return subprocess.run(diff).returncode


if __name__ == "__main__":
    sys.exit(main())
