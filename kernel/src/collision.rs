//! Wide-SIMD separating-axis collision testing, following the approach
//! described in Erin Catto's "SIMD for Collision" (box2d.org, 2026-07-18):
//! process multiple candidate separating axes in one SIMD lane, rather than
//! one axis at a time.
//!
//! NOT currently wired into kernel8's boot sequence — kept here as a
//! tested-in-isolation, not-integrated module, same standard as
//! shared/tensor_compress.py's negative result in the sibling ML pipeline
//! project: real code, really tested, honestly not shipped where it didn't
//! work.
//!
//! What's actually true: this exact SSE2 intrinsic pattern (runtime,
//! non-constant f32 inputs, `_mm_set_ps`/`_mm_sub_ps`/`_mm_cmple_ps`/
//! `_mm_movemask_ps`) compiles and runs correctly in an isolated minimal
//! `#![no_std]` crate targeting `x86_64-unknown-none`, at both `--release`
//! and `-C opt-level=2` in dev mode. Integrated into this actual kernel
//! binary (built via the unstable `-Z bindeps` artifact-dependency
//! mechanism this project already relies on for the bootloader), it
//! reproducibly crashes rustc itself: `rustc-LLVM ERROR: Do not know how
//! to split the result of this operator!`, persisting through opt-level 2
//! and 3. Also confirmed along the way: `-Z bindeps` artifact builds don't
//! forward ANY Cargo.toml profile setting to the artifact's rustc
//! invocation (`[profile.dev]`, `[profile.release]`, and
//! `[profile.*.build-override]` were all tried and verified via `cargo
//! build -v` to add no `-C opt-level` flag at all) — only `rustflags` in
//! `.cargo/config.toml` actually reaches it, which is how the opt-level
//! bisection above was even possible to test.
//!
//! Not investigated further: whether this is a real rustc/LLVM bug
//! specific to this target+bindeps combination (plausible — `-Z bindeps`
//! is genuinely immature), or something else in how this file's code
//! specifically differs from the isolated repro. That's a real follow-up,
//! not a "coming soon" — worth an actual upstream rust-lang issue with the
//! minimal repro before trying to wire this back in.
//!
//! Separately, and independent of the crash: the design intent stands.
//! SSE2 is scoped to only this `#[target_feature]`-tagged function, not
//! the whole crate (which stays soft-float/no-SIMD, matching
//! `x86_64-unknown-none`'s default) — specifically so this stays safe to
//! use even though this kernel's `x86-interrupt` handlers don't save/
//! restore XMM state. As long as no OTHER code path touches XMM, an
//! interrupt firing mid-function here can't clobber anything, because the
//! handler never reads or writes those registers.
use core::arch::x86_64::{__m128, _mm_cmple_ps, _mm_movemask_ps, _mm_set_ps, _mm_sub_ps};

/// An axis-aligned box: center + half-extents, one per dimension x/y/z.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub center: [f32; 3],
    pub half_extent: [f32; 3],
}

/// Tests all 3 axis-aligned candidate separating axes AT ONCE in a single
/// SIMD register (padded to 4 lanes — SSE2 has no 3-wide vector), instead of
/// three sequential scalar comparisons. This is the actual "wide SIMD"
/// technique: one vectorized compare covers every axis, one pass.
///
/// # Safety
/// Requires the CPU to support SSE2, which is mandatory baseline for every
/// real x86_64 CPU (part of the architecture's own guarantee, not optional
/// hardware) — there is no CPUID check needed the way there would be for a
/// truly optional extension like AVX2.
#[target_feature(enable = "sse2")]
pub unsafe fn aabb_overlap(a: &Aabb, b: &Aabb) -> bool {
    // distance between centers on each axis, 4th lane unused (set to 0, and
    // trivially "not separating" so it never affects the result).
    let delta: __m128 = _mm_set_ps(
        0.0,
        (a.center[2] - b.center[2]).abs(),
        (a.center[1] - b.center[1]).abs(),
        (a.center[0] - b.center[0]).abs(),
    );
    let combined_extent: __m128 = _mm_set_ps(
        f32::MAX, // 4th lane: force "within range" so it can't cause a false separation
        a.half_extent[2] + b.half_extent[2],
        a.half_extent[1] + b.half_extent[1],
        a.half_extent[0] + b.half_extent[0],
    );
    // Separating Axis Test: boxes overlap on an axis iff |delta| <= sum of
    // half-extents. All 3 axes tested in the single vectorized compare below.
    let within_range = _mm_sub_ps(delta, combined_extent); // <= 0 means overlapping on that axis
    let cmp = _mm_cmple_ps(within_range, _mm_set_ps(0.0, 0.0, 0.0, 0.0));
    let mask = _mm_movemask_ps(cmp);
    // Overlap on ALL 3 real axes (bits 0,1,2) means the boxes intersect —
    // if separated on even one axis, that's a valid separating axis (SAT).
    (mask & 0b0111) == 0b0111
}
