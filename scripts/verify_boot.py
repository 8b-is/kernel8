"""Validate serial milestones; passing a synthetic log does not prove a real boot."""
import re
import sys
from pathlib import Path

REQUIRED = (
    "kernel8: booted.",
    "survived a breakpoint exception. the IDT is real.",
    "mapped a new page, wrote 0xC0FFEE, read back 0xC0FFEE.",
    "paging works.",
    "heap-allocated Box: 1991",
    "heap-allocated Vec of 500 elements, sum = 124750",
    "the heap works.",
    "task a: 1", "task b: 1", "task a: 2", "task b: 2",
    "cooperative multitasking works (interleaved output above, not sequential).",
    "interrupts enabled. counting quanta (timer ticks) before free-running:",
)

def validate(text):
    if re.search(r"kernel panic|panicked at|triple fault|double fault", text, re.I):
        raise ValueError("panic or fault in boot log")
    if text.count("kernel8: booted.") != 1:
        raise ValueError("expected exactly one boot")
    lines = [line.strip() for line in text.splitlines()]
    cursor = 0
    for milestone in REQUIRED:
        try:
            cursor = lines.index(milestone, cursor) + 1
        except ValueError:
            raise ValueError("missing or out-of-order milestone: " + milestone) from None
    tasks = [line for line in lines if line.startswith("task ")]
    if tasks != ["task a: 1", "task b: 1", "task a: 2", "task b: 2"]:
        raise ValueError("unexpected task output or ordering")
    # Interrupts remain enabled: dots may arrive between the count snapshot
    # and printing it. Validate the threshold, not an exact dot/count match.
    tail = "\n".join(lines[cursor:])
    match = re.match(r"(\.+)counted (\d+) timer quanta — discrete, not interpolated\.\n\.*free-running now, one '\.' per quantum:", tail)
    if not match or int(match[2]) < 20:
        raise ValueError("missing timer milestone or count below threshold")

if __name__ == "__main__":
    try:
        validate(Path(sys.argv[1]).read_text())
    except (ValueError, OSError, IndexError) as exc:
        sys.exit("boot validation failed: " + str(exc))
    print("serial milestones verified; task order and measured values checked")
