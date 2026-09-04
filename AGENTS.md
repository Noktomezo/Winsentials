# Project rules

## Rust completion gate

Before marking any Rust-related task complete, run and pass:

```text
cargo fmt --all --check
cargo check
cargo clippy -- -D warnings
```

If the affected code has relevant tests, run them too. This includes tests annotated with `#[gpui::test]`; use the repository's documented test command when it differs from `cargo test`. Report any check that cannot run or does not pass.

## Project structure

Keep the source tree in a lightweight, Rust-adapted FSD shape as the application grows:

```text
src/
  app/       # bootstrap and top-level composition
  pages/     # full screens
  widgets/   # substantial reusable UI sections
  features/  # user-facing interactions and workflows
  entities/  # domain models and their UI
  shared/    # UI primitives, theme, assets, and general utilities
```

Dependencies flow downward: `app -> pages -> widgets -> features -> entities -> shared`. Keep `main.rs` as the bootstrap entry point. Create a layer only when it contains real code; small features may remain in one file until splitting improves navigation.

## Reuse and DRY: promote before duplicating

Before implementing a general-purpose capability, check in order: existing project code, the Rust standard library, native platform APIs, installed dependencies, and established crates. Prefer a mature, maintained crate when it solves the actual requirement and fits the project's license, platform, MSRV, and dependency constraints. Write a custom implementation only for project-specific behavior or a concrete mismatch, and record the reason when it is not obvious. Keep trivial code local when adding a dependency would cost more than the implementation.

Apply **Don't Repeat Yourself (DRY)** with active promotion:
- **Zero parallel duplicates**: Never copy-paste or write ad-hoc parallel variations of recurring UI elements (such as buttons, badges, chips, sparklines, cards), styling snippets, or domain logic across pages and widgets.
- **Promote to library grade**: If an element or pattern appears more than once—or clearly represents a reusable primitive—do not keep it inline. Refactor and elevate it into a production-grade component in `shared/ui` (or a utility in `shared/` / `entities/`) with a complete visual, behavioral, and motion contract, then consume it cleanly at all call sites.
- **Duplication audit (`jscpd`)**: To monitor and catch copy-pasted UI fragments and boilerplate across crates, run:
  ```text
  bunx jscpd crates --min-lines 10 --reporters console --summary
  ```
  Keep the overall codebase duplication low (target < 7%) by factoring out repeated layout blocks, card templates, and repetitive iteration logic.

## Library-grade GPUI components

Treat reusable GPUI components in `shared/ui` (and interactive sub-elements across `widgets` and `features`) as production-grade library components, comparable to a shadcn component with React Aria-quality ergonomics. Use shadcn as the visual baseline: restrained sizing, spacing, radii, icon proportions, state styling, and component density, adapted to the product theme and native GPUI constraints.

Each component owns its full visual, behavioral, and **motion contract**:
- **Variations and slots**: Typed props, variants, sizes, slots, and callbacks. Never expose raw styling escape hatches or fork markup at use sites. Promote new legitimate variants into the component API.
- **States and ergonomics**: Hit areas, focus rings, keyboard navigation, disabled state, and accessibility.
- **In-component motion**: Micro-interactions (smooth hover fades, active press feedback, focus transitions, toggle slides, icon rotations) belong *inside* the component contract. Ad-hoc, abrupt styling jumps (such as instant `.hover(|s| s.bg(...))` without transition continuity) violate the library contract. Call sites only configure and compose.

## Motion-first GPUI

Treat motion as an intrinsic part of the user experience rather than optional polish. GPUI is abrupt by default; every state, layout, visibility, scroll, and micro-interaction must feel continuous and grounded:

- **Component-level continuity**: Hover, press, selection, and activation states must transition smoothly using shared duration and easing primitives. Never hard-swap interactive colors or bounds when a continuous transition preserves spatial context.
- **Structural motion**: Prefer interruptible transitions, smooth scrolling, and proportional morphing over sudden pops. Direct manipulation must track input with zero lag, and motion must never delay an action or block user throughput.
- **Lifecycle hygiene**: Keep animations event-driven or reactive. Never spawn unbounded animation loops or active tick handlers inside standard `render()` callbacks that force perpetual VSync redraws.
- **Reduced motion**: Respect system accessibility preferences unconditionally. When reduced motion is enabled, replace large translations, smooth scrolling, spring overshoots, and morphs with immediate swaps or subtle fades. Preserve hierarchy, focus, and feedback with less motion, not slower motion.

## Measured performance and concurrency

Measure before optimizing. Profile and benchmark release builds against an explicit latency, throughput, or memory target, then improve algorithms, data layout, allocations, and batching before adding concurrency. Keep before/after benchmark evidence for nontrivial performance work.

Classify work before choosing an executor:

- Use Rayon for measured CPU-bound data-parallel work with independent items and enough input to amortize scheduling; keep small workloads sequential and avoid nested thread-pool oversubscription.
- Use async for concurrent I/O. Prefer GPUI's foreground and background executors for ordinary application work; add Tokio when an integration needs its runtime, networking, timers, or task ecosystem. Own one runtime at the application boundary rather than creating runtimes per component.
- Move blocking APIs and mixed CPU/I/O work to `spawn_blocking`, `cx.background_spawn`, or a dedicated bounded worker. Use a dedicated OS thread only for thread-affine or long-lived blocking loops.
- Build long-lived pipelines from bounded tasks and channels with backpressure, cancellation, and capped fan-out. Prefer ownership and message passing over shared mutable state; scope locks tightly and never hold a lock across `.await`.

Keep GPUI render, prepaint, paint, and input callbacks free of I/O, blocking calls, long locks, and heavy computation. Compute off-thread, return results to the foreground executor, update entities there, and coalesce state changes into the fewest necessary `notify` calls and animation frames.

## Error handling

Use both crates by layer. Public APIs in `shared`, `entities`, reusable components, and domain boundaries return typed `thiserror` enums when callers may inspect, recover from, or present distinct failures. Application orchestration and top-level tasks may return `anyhow::Result`; add `anyhow::Context` at I/O and subsystem boundaries, and convert typed errors to `anyhow` only at the application boundary. Preserve each error's `source` instead of flattening failures into strings.

Use `Option` only for normal absence and `Result` for expected failure. Reserve `panic!`, `assert!`, `unreachable!`, and `expect` for documented programmer invariants; handle user input, I/O, network, parsing, and cancellation without panicking. Never discard a `Result` from a detached GPUI task: map it to explicit UI state, log the technical chain once, and show a user-facing message without internal details. Treat cancellation as control flow, and use bounded retries only for idempotent transient failures.
