#!/usr/bin/env python3
"""Find the first cycle where gb_emu diverges from a reference core.

The reference (SameBoy) samples once per instruction; gb_emu samples every M-cycle. The two are
joined on the absolute M-cycle column, so an instruction that takes the wrong number of cycles
shows up as state landing on the wrong cycle -- which is what the mooneye timing tests measure.

Two classes of difference get reported separately, because they want different fixes:

  systematic  a column that disagrees on essentially every sample. Almost always a read-back
              masking bug (unused register bits reading 0 instead of 1) rather than a timing bug.
              Reported once, not once per cycle, so it does not bury everything else.

  divergence  the first cycle where a column that had been agreeing stops agreeing. This is the
              one you debug.

Usage:
  python scripts/trace_diff.py <reference.trace> <gb_emu.trace> [--context=N] [--offset=N]
                              [--from-cycle=N] [--ignore=col,col]

--from-cycle is the one you reach for most: the boot ROM runs before any cartridge code, so a
divergence in it is reported identically for every ROM and hides everything the test itself does.
Pass a cycle past the boot handoff to compare only the ROM under test.
"""
import sys
from collections import OrderedDict

# gb_emu does not expose AF/SP, so those reference columns are carried for display only.
REF_COLUMNS = ["cycle", "pc", "af", "bc", "de", "hl", "sp", "ly", "stat", "div", "tima", "if", "ie"]
GB_COLUMNS = ["cycle", "pc", "bc", "de", "hl", "ly", "stat", "div", "tima", "if", "ie"]
COMPARED = ["pc", "bc", "de", "hl", "ly", "stat", "div", "tima", "if", "ie"]

# gb_emu's get_pc() is a fetch pointer rather than the address of the instruction being executed,
# so it runs ahead of the reference's pc by a variable amount. Comparing it would report a
# divergence on nearly every line; it is shown in context instead.
DISPLAY_ONLY = {"pc"}


def load(path, columns):
    rows = {}
    order = []
    with open(path) as handle:
        for line in handle:
            parts = line.split()
            if len(parts) != len(columns):
                continue
            row = dict(zip(columns, parts))
            cycle = int(row["cycle"])
            row["cycle"] = cycle
            rows[cycle] = row
            order.append(row)
    return rows, order


def calibrate(ref_order, gb_rows, max_offset=4, sample=300):
    """Pick the cycle offset that best lines the two traces up.

    Sampling conventions differ (the reference samples before an instruction, gb_emu before a
    tick), so a small constant skew is expected and is not itself a bug.
    """
    best, best_score = 0, -1
    for offset in range(max_offset + 1):
        score = 0
        for ref in ref_order[:sample]:
            gb = gb_rows.get(ref["cycle"] + offset)
            if not gb:
                continue
            score += sum(1 for col in COMPARED if col not in DISPLAY_ONLY and ref[col] == gb[col])
        if score > best_score:
            best, best_score = offset, score
    return best


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    flags = {a.split("=")[0]: a.split("=")[1] for a in sys.argv[1:] if a.startswith("--") and "=" in a}
    if len(args) < 2:
        print(__doc__)
        return 2
    context = int(flags.get("--context", 6))
    from_cycle = int(flags.get("--from-cycle", 0))
    ignored = {c.strip() for c in flags.get("--ignore", "").split(",") if c.strip()}

    ref_rows, ref_order = load(args[0], REF_COLUMNS)
    gb_rows, _ = load(args[1], GB_COLUMNS)
    if not ref_order or not gb_rows:
        print("one of the traces is empty")
        return 1

    offset = int(flags["--offset"]) if "--offset" in flags else calibrate(ref_order, gb_rows)

    paired = [
        (r, gb_rows[r["cycle"] + offset])
        for r in ref_order
        if r["cycle"] >= from_cycle and (r["cycle"] + offset) in gb_rows
    ]
    if not paired:
        print("traces do not overlap; is one of them too short?")
        return 1

    # A column that disagrees almost everywhere is a masking bug, not a timing bug.
    systematic = OrderedDict()
    for col in COMPARED:
        if col in DISPLAY_ONLY or col in ignored:
            continue
        misses = [(r, g) for r, g in paired if r[col] != g[col]]
        if len(misses) > len(paired) * 0.9:
            systematic[col] = misses[0]

    print(f"reference samples: {len(ref_order)}   gb_emu cycles: {len(gb_rows)}   "
          f"paired: {len(paired)}   cycle offset: +{offset}"
          + (f"   from cycle {from_cycle}" if from_cycle else "")
          + (f"   ignoring {', '.join(sorted(ignored))}" if ignored else ""))

    if systematic:
        print("\nsystematic differences (every sample -- look at read-back masking, not timing):")
        for col, (ref, gb) in systematic.items():
            print(f"  {col:<5} reference={ref[col]}  gb_emu={gb[col]}  (from cycle {ref['cycle']} onward)")

    live = [c for c in COMPARED if c not in DISPLAY_ONLY and c not in systematic and c not in ignored]
    first = None
    for index, (ref, gb) in enumerate(paired):
        bad = [c for c in live if ref[c] != gb[c]]
        if bad:
            first = (index, ref, gb, bad)
            break

    if not first:
        print(f"\nno divergence in {len(live)} compared columns over {len(paired)} samples: {', '.join(live)}")
        return 0

    index, ref, gb, bad = first
    print(f"\nfirst divergence at M-cycle {ref['cycle']} in: {', '.join(bad)}")
    for col in bad:
        print(f"  {col:<5} reference={ref[col]}  gb_emu={gb[col]}")

    print(f"\ncontext (reference pc is the instruction about to run; gb_emu pc is a fetch pointer):")
    header = f"  {'cycle':>8}  {'ref pc':>6} {'ref af':>6} {'ref sp':>6}  {'gb pc':>6}  " + \
             "  ".join(f"{c:>5}" for c in live)
    print(header)
    for r, g in paired[max(0, index - context):index + context + 1]:
        mark = "->" if r["cycle"] == ref["cycle"] else "  "
        cells = "  ".join(
            (f"{r[c]}/{g[c]}" if r[c] != g[c] else f"{r[c]:>5}") for c in live
        )
        print(f"{mark}{r['cycle']:>8}  {r['pc']:>6} {r['af']:>6} {r['sp']:>6}  {g['pc']:>6}  {cells}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
