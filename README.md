# vakedkernel

A real, bootable, 100% Rust x86_64 kernel — the honest first milestone
toward a Unix-like OS, not a finished one. Built on the actual
[rust-osdev](https://github.com/rust-osdev) ecosystem (`bootloader`,
`bootloader_api`, `uart_16550`), following the well-established `blog_os` /
Redox lineage rather than inventing bare-metal plumbing from scratch.

## What's actually true right now

Verified by hand, not asserted:

```
$ cargo run -- --no-reboot
...
vakedkernel: booted.
physical_memory_offset: None
this is a real boot, not a simulation of one.
```

That's the whole kernel today: it boots via BIOS in QEMU, brings up the
serial port (COM1), and prints. `physical_memory_offset: None` is expected —
the physical-memory-mapping bootloader feature isn't requested yet, not a
bug.

## What's not built yet (real roadmap, not shipped)

In the order each actually unblocks the next:

1. **GDT + IDT** — a proper Global Descriptor Table and interrupt handlers
   (double-fault handler first, so a bug crashes with a message instead of
   a silent triple-fault reboot).
2. **Physical + virtual memory management** — frame allocator over the
   bootloader's memory map, paging.
3. **Heap allocation** — a global allocator, so `alloc::*` (Vec, Box, etc.)
   works in kernel code.
4. **A minimal process/scheduler concept** — this is where "Unix design"
   actually starts meaning something, not just an aesthetic.
5. **A syscall interface** — the actual userland boundary.
6. **[rustybox](https://github.com/peterlodri-sec/rustybox) as the
   userland** — the point of the whole exercise. RustyBox already gives a
   real, working BusyBox-equivalent CLI toolkit in Rust; this kernel's job
   is to become something rustybox can run *on*, not to reinvent a
   userland.

## Running it

```sh
rustup target add x86_64-unknown-none
rustup component add llvm-tools-preview
cargo run -- --no-reboot   # -Z bindeps requires the nightly pinned in rust-toolchain.toml
```

Requires QEMU (`brew install qemu` / your distro's package). `--no-reboot`
stops QEMU cleanly on triple fault instead of rebooting into a loop — useful
once step 1 above is unstable code, not needed for the current kernel which
never faults.

## Why not a custom target spec

Older Rust-OS tutorials use a hand-written `x86_64-*.json` target. Current
`bootloader_api` targets the built-in Tier-2 `x86_64-unknown-none`, which is
simpler and is what this repo uses — one less hand-maintained JSON file to
get subtly wrong.
