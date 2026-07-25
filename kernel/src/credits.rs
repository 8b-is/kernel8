//! None of this exists in a vacuum.
//!
//! Ken Thompson and Dennis Ritchie built Unix at Bell Labs starting in 1969
//! — the process model, the hierarchical filesystem, "everything is a file,"
//! pipes, the whole shape of what an operating system even is that every
//! line in this repo still assumes without question.
//!
//! Linus Torvalds started Linux in 1991 as a hobby project, said so
//! explicitly, and then spent the following decades proving that an open,
//! freely reusable Unix-like kernel could actually run the world's
//! infrastructure. rustybox, this kernel's intended eventual userland,
//! exists downstream of that lineage too (BusyBox → Linux userspace).
//!
//! This kernel is a beginner's project standing on both of those
//! shoulders, not a peer to them. Printed once at boot, not because they'll
//! ever see it, but because it should be said out loud in the actual code,
//! not just in a commit message that rots.

pub fn print() {
    crate::println!("---");
    crate::println!("Unix: Ken Thompson & Dennis Ritchie, Bell Labs, 1969.");
    crate::println!("Linux: Linus Torvalds, 1991, and everyone since.");
    crate::println!("This kernel exists downstream of both. Thank you.");
    crate::println!("---");
}
