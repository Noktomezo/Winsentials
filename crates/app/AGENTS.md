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

## Reuse before build

Before implementing a general-purpose capability, check in order: existing project code, the Rust standard library, native platform APIs, installed dependencies, and established crates. Prefer a mature, maintained crate when it solves the actual requirement and fits the project's license, platform, MSRV, and dependency constraints. Write a custom implementation only for project-specific behavior or a concrete mismatch, and record the reason when it is not obvious. Keep trivial code local when adding a dependency would cost more than the implementation.

## Library-grade GPUI components

Treat reusable GPUI components in `shared/ui` as production-grade library components, comparable to a shadcn component with React Aria-quality behavior. Use shadcn as the approximate visual and compositional baseline: restrained sizing, spacing, radii, icon proportions, state styling, and component density, adapted to the product theme and native GPUI constraints rather than copied literally. Each component owns its complete visual and interaction contract: variants, sizes, content slots, state, events, focus, keyboard behavior, disabled state, accessibility, theme tokens, and hit areas.

Expose supported variation through deliberate typed props, builders, variants, slots, and callbacks so call sites only configure and compose the component. When a use site needs a legitimate new variation, promote it into the component API or keep the product-specific composition in a higher FSD layer. Keep styling and behavior fixes inside the component instead of forking its markup or patching its internals at the use site. Prefer a small coherent API over a raw style escape hatch.

## Motion-first GPUI

Treat motion as part of the interaction contract rather than optional polish. GPUI is abrupt by default, so explicitly design continuity for meaningful state, layout, visibility, scroll, content, and icon transitions. Prefer interruptible motion, smooth scrolling, coherent easing, and morphing over hard swaps when they improve orientation or feedback. Reusable components own their motion behavior and expose deliberate typed variants; share common timing and easing primitives instead of tuning animations at call sites. Direct manipulation must track input without lag, and motion must never delay an action or hide its result.

The system reduced-motion preference overrides motion-first behavior. When it is enabled, use immediate state changes or minimal fades; disable smooth scrolling, autoplay, spring overshoot, parallax, large transforms, and nonessential morphing. Preserve focus, hierarchy, feedback, and hit targets, and use less motion rather than slower motion.

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
