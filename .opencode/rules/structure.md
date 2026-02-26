**Project Folder Structure Rule (Tauri 2.x + Rust Backend + TypeScript Frontend)**

You ALWAYS follow this exact folder structure in the project. It guarantees scalability, alignment between frontend and backend, and easy onboarding.

**Root Structure**
/
├── src/ # Frontend — TypeScript (FSD architecture)
├── src-tauri/ # Backend — Rust + Tauri core
│ ├── src/
│ ├── Cargo.toml
│ ├── tauri.conf.json
│ ├── capabilities/
│ └── build.rs
├── package.json
├── tsconfig.json
└── ...

**Frontend — Feature-Sliced Design (FSD)**
src/
├── app/ # Application-level: providers, routing, global styles, store init
├── pages/ # Full pages / routes (one folder per page)
├── widgets/ # Complex independent UI blocks (composed of features + entities)
├── features/ # Business features & use-cases (main place for new logic)
├── entities/ # Business entities (User, Note, Project, Settings…)
├── shared/
│ ├── ui/ # UI kit, components, design system
│ ├── api/ # Tauri invoke wrappers + API layer
│ ├── lib/ # Hooks, utils, helpers
│ ├── config/
│ ├── constants/
│ └── types/
└── index.tsx

**Backend (Rust) — Vertical Feature Slices**
src-tauri/src/
├── main.rs
├── lib.rs
├── commands/ # Thin layer: ONLY #[tauri::command] handlers
├── core/ # Global infrastructure (not tied to any feature)
│ ├── config.rs
│ ├── error.rs # AppError + thiserror
│ ├── state.rs # Tauri managed state
│ ├── database.rs
│ └── constants.rs
├── features/ # Main business logic — aligned with frontend features/
│ ├── auth/
│ │ ├── commands.rs
│ │ ├── service.rs # Business logic
│ │ ├── repository.rs # Data access
│ │ └── types.rs # Domain models
│ ├── notes/
│ ├── settings/
│ └── ...
├── shared/ # Truly shared code
│ ├── models.rs
│ ├── types.rs
│ └── utils.rs
└── types.rs # Common types (if needed outside features)

**Mandatory Principles**

1. New major functionality → add under `features/` on BOTH sides (keep names identical when possible).
2. `commands/` must stay thin — delegate everything to `features/*/service.rs`.
3. Never put business logic directly in `main.rs` or root-level files.
4. When creating a new feature/entity, always specify exact folder paths in your answer.
5. For small projects (<6 features) you may flatten `features/` into `services/`, but switch to vertical slices as soon as the project grows.

When generating code, reviewing PRs or planning architecture — always reference and respect this structure.
