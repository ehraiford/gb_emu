#!/usr/bin/env bash
# Builds the SameBoy-backed reference tracer used by scripts/trace_diff.py.
#
#   ./tools/reference_tracer/build.sh
#
# Clones SameBoy next to this script (gitignored), extracts gb_emu's own boot ROM so both cores
# boot from identical bytes, and links a headless tracer. Needs clang; MSVC's cl.exe will not do,
# since SameBoy's core relies on GNU extensions.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
SAMEBOY="$HERE/sameboy"

CLANG="${CLANG:-}"
if [[ -z "$CLANG" ]]; then
    if command -v clang >/dev/null 2>&1; then
        CLANG="$(command -v clang)"
    elif [[ -x "/c/Program Files/LLVM/bin/clang.exe" ]]; then
        CLANG="/c/Program Files/LLVM/bin/clang.exe"
    else
        echo "clang not found; set CLANG=/path/to/clang" >&2
        exit 1
    fi
fi
echo "using clang: $CLANG"

if [[ ! -d "$SAMEBOY" ]]; then
    echo "cloning SameBoy..."
    git clone --depth 1 https://github.com/LIJI32/SameBoy.git "$SAMEBOY"
fi

# Both cores must run the same boot ROM or the traces diverge before the cartridge starts.
echo "extracting boot ROM from src/onboard_memory/bootrom.rs..."
python "$HERE/extract_bootrom.py" "$ROOT/src/onboard_memory/bootrom.rs" "$HERE/dmg_boot.bin"

# Notes on the flags, all of which were needed to get the core through clang on Windows:
#   -DGB_INTERNAL      defs.h and the real GB_sample_t are gated behind it
#   -I Windows         SameBoy's own shims for getline/vasprintf/ssize_t
#   -Drandom=rand      what SameBoy's own Windows target does
#   utf8_compat.c      excluded: it redefines fopen and collides with libucrt
WIN_SOURCES=$(ls "$SAMEBOY"/Windows/*.c 2>/dev/null | grep -v utf8_compat || true)

"$CLANG" -O2 -w -std=gnu11 \
    -D_GNU_SOURCE -D_USE_MATH_DEFINES -DGB_INTERNAL -Drandom=rand \
    -DGB_VERSION='"trace"' -DGB_COPYRIGHT_YEAR='"2025"' \
    ${WIN_SOURCES:+-I "$SAMEBOY/Windows"} -I "$SAMEBOY" \
    "$HERE/tracer.c" "$SAMEBOY"/Core/*.c $WIN_SOURCES \
    -o "$HERE/tracer.exe"

echo "built $HERE/tracer.exe"
