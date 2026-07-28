# kernel8 — v42.0 · 0.2.0

**CPU (hearth) of the ayeOS mesh.** x86_64 Rust kernel. Bootable. Cooperative async. Paging. Heap.

Part of the [ayeOS](https://github.com/8b-is/ayeos) ternary inference mesh:
| Layer | Project |
|-------|---------|
| **CPU (hearth)** | kernel8 (this repo) |
| GPU (brain) | [MLX-QUANT](https://github.com/8b-is/MLX-QUANT) |
| Coord | [vaked](https://github.com/8b-is/vaked) |
| Daemon | [ayeOS](https://github.com/8b-is/ayeos) |
| Portal | [axiomquant.org](https://axiomquant.org) |

[![CI](https://github.com/8b-is/kernel8/actions/workflows/ci.yml/badge.svg)](https://github.com/8b-is/kernel8/actions/workflows/ci.yml)

A real, bootable, 100% Rust x86_64 kernel. Built on the actual
[rust-osdev](https://github.com/rust-osdev) ecosystem (`bootloader`,
`bootloader_api`, `x86_64`, `uart_16550`, `pic8259`, `pc-keyboard`,
`linked_list_allocator`), following the well-established `blog_os`
lineage — every non-trivial piece adapted from real, current reference
implementations (checked against docs.rs at time of writing, not
reconstructed from memory), not invented bare-metal plumbing.

Formerly `vakedkernel` (`peterlodri-sec/vakedkernel`) — moved here to
[8b-is](https://github.com/8b-is) and renamed.

**"v42.0" is a Douglas Adams joke, not a claim of 42 real releases.** The
actual milestones are the ones listed below — same discipline as the "v1.0"
note it replaces: the number doesn't mean more than what's verified under
it. What changed since the last checkpoint: a cooperative async/await
executor, and one honestly-documented negative result (SIMD collision
detection — real bug hit, not shipped, see below). It does not mean
UNIX-compliant. UNIX® conformance is a specific certification administered
by The Open Group against the Single UNIX Specification and its test
suites, granted to particular registered products and releases — it isn't
a property of operating-system families in the abstract, and this project
has neither attempted nor claimed that certification. What follows is
Unix-inspired, not UNIX-certified, and real.

## What's actually true right now

Verified by hand, running it, not asserted. Boot output (serial, `-serial
stdio -display none`):

```
kernel8: booted.
Unix: Ken Thompson & Dennis Ritchie, Bell Labs, 1969.
Linux: Linus Torvalds, 1991, and everyone since.
This kernel exists downstream of both. Thank you.
gdt + idt loaded.
EXCEPTION: BREAKPOINT
survived a breakpoint exception. the IDT is real.
mapped a new page, wrote 0xC0FFEE, read back 0xC0FFEE.
paging works.
heap-allocated Box: 1991
heap-allocated Vec of 500 elements, sum = 124750
the heap works.
task a: 1
task b: 1
task a: 2
task b: 2
cooperative multitasking works (interleaved output above, not sequential).
interrupts enabled. counting quanta (timer ticks) before free-running:
....................counted 20 timer quanta — discrete, not interpolated.
free-running now, one '.' per quantum:
....................................
```

Concretely, what each line is actual proof of, not a claim:

- **Boots** via BIOS in QEMU, brings up serial (COM1).
- **GDT + IDT + double-fault handler** — a real breakpoint exception (`int3`)
  is triggered and caught by the actual handler, which prints and returns
  cleanly. (Hit a real bug getting here: reloading only `CS` after building
  a new GDT — the tutorial's own base case — left `SS` pointing at a stale
  index that collided with the new GDT's TSS descriptor slot, double-faulting
  on the very first interrupt return. Fixed by adding an explicit
  `kernel_data_segment` and reloading `SS` too.)
- **Paging** — a real frame is allocated, a brand-new virtual page is mapped
  to it, a known value is written through that mapping and read back
  correctly, and `translate_addr` confirms the mapping independently.
- **Heap allocation** (`linked_list_allocator`) — a real `Box` and a `Vec`
  that grows past its initial capacity (forcing a reallocation through the
  actual allocator), value-checked (`sum(0..500) == 124750`), not just
  "it compiled."
- **Cooperative multitasking** (async/await, `SimpleExecutor`) — two tasks,
  each yielding once mid-execution, produce interleaved output (`a1, b1,
  a2, b2`) rather than sequential (`a1, a2, b1, b2`) — the only way to tell
  "the executor interleaves" from "the executor just runs tasks one after
  another." Deliberately cooperative, not preemptive multi-process
  scheduling with hand-written context-switch assembly — that's a
  separate, larger undertaking even blog_os defers.
- **Hardware interrupts** (PIC remap via `pic8259`, timer) — continuous,
  live ticks, confirmed by watching them actually happen, including correct
  PIC EOI signaling (get that wrong and the PIC stops sending interrupts
  after the first one).
- **Quantum tick counter** (`interrupts::ticks()`) — a real `AtomicU64`
  incremented once per timer interrupt, no float, no interpolation. Per
  [pocoo.vaked.dev/posts/2026-07-26-quantum-means-counting](https://pocoo.vaked.dev/posts/2026-07-26-quantum-means-counting):
  *quantum* is Latin *quantus*, "how much," and "how much" is always a
  count — the discrete is the floor, the continuum (the usual `f64`
  elapsed-time approach to OS timekeeping) is the derived idealization
  built on top of it. Boot proves the counter is exact, not estimated: it
  waits for `ticks() == 20`, and the number of `.` printed in that stretch
  is observably, exactly 20 — see the boot log above.

- **Keyboard input, verified live** — same PIC/IDT/EOI machinery as the
  timer, implemented against `pc-keyboard` 0.9.0's *current* API
  (`PS2Keyboard`, checked against docs.rs — the crate renamed its main
  struct since older tutorials were written). No display was needed to
  prove this after all: QEMU's HMP monitor can inject real PS/2 scancodes
  headlessly (`-monitor telnet:...` + `sendkey a`), which drives the exact
  same hardware-interrupt path a real keyboard would — port 0x60 read,
  IDT dispatch, `PS2Keyboard` decode, print. Injecting `a`, `b`, `c` and
  reading the serial log back shows `abc` inline in the timer-tick dot
  stream, exactly where the keys were sent.

**Frame allocator never frees.** `BootInfoFrameAllocator` (the blog_os
reference implementation this follows) only ever advances forward through
usable physical frames — there is no `deallocate_frame`. Fine for
everything built so far (a few dozen frames total: the heap, one demo
page). Real userland work — an ELF loader mapping process segments, a
scheduler tearing down exited processes — will exhaust physical memory
under this allocator eventually, since nothing is ever returned to the
pool. Flagged here, not fixed pre-emptively: the actual free-list
bookkeeping this needs is naturally part of the Ring 3 / process
lifecycle milestone below, not something to bolt on speculatively before
anything exists that would exercise it.

**Wide-SIMD collision detection was attempted and did not ship** — see
`kernel/src/collision.rs`. Real technique (Erin Catto's "SIMD for
Collision," box2d.org 2026-07-18: process multiple separating-axis tests
in one SIMD lane), real code, verified working in an isolated minimal
crate on this exact target. Integrated into this kernel's actual build
(via the unstable `-Z bindeps` artifact-dependency mechanism this project
already relies on for the bootloader), it reproducibly crashes rustc:
`rustc-LLVM ERROR: Do not know how to split the result of this operator!`,
persisting through opt-level 2 and 3. Not wired into the boot sequence.
Worth an upstream rust-lang issue with the minimal repro before retrying.

## What's not built (real roadmap, not shipped, not silently implied)

Each of these is roughly as much work as everything above combined —
listed honestly as separate, large milestones, not squeezed into the
version number to make it look bigger:

1. **Ring 3 (usermode) + a syscall ABI.** The actual userland boundary —
   privilege transitions, a real syscall table, TSS-based stack switching
   for the return path.
2. **An ELF loader** for running independent userspace programs.
3. **[rustybox](https://github.com/8b-is/rustybox) as the
   userland** — the actual point of the whole exercise. RustyBox already
   gives a real, working BusyBox-equivalent CLI toolkit in Rust (and now
   also a `spherepop` applet — a deterministic event-log kernel, unrelated
   to this repo's hardware-kernel track); this kernel's job is to become
   something rustybox can run *on*.

## Running it

```sh
rustup target add x86_64-unknown-none
rustup component add llvm-tools-preview
cargo run -- --no-reboot   # -Z bindeps requires the nightly pinned in rust-toolchain.toml
```

Requires QEMU (`brew install qemu` / your distro's package). `--no-reboot`
stops QEMU cleanly on a triple fault instead of rebooting into a loop.

## Why not a custom target spec

Older Rust-OS tutorials use a hand-written `x86_64-*.json` target. Current
`bootloader_api` targets the built-in Tier-2 `x86_64-unknown-none`, which is
simpler — one less hand-maintained JSON file to get subtly wrong.

## On the `wild` linker

Tried wiring [`wild`](https://github.com/wild-linker/wild) in as this
kernel's linker (forked at
[peterlodri-sec/wild](https://github.com/peterlodri-sec/wild)). Its own
README only documents `-unknown-linux-gnu`/illumos userspace targets — no
mention of bare-metal/freestanding support anywhere. Got as far as
reaching the actual link step (confirming the binary runs and is invoked
correctly) before hitting real argument-compatibility gaps between what
rustc's freestanding-target linker invocation emits and what wild currently
accepts. Reverted to the default linker rather than keep guessing flag
combinations that could silently produce a subtly-broken kernel. This is
genuinely unexplored territory for wild's own project, not a quick config
fix — worth a real upstream issue, not a hack here.
