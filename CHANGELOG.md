# Changelog

All notable changes to Winsentials are documented in this file.

## [0.7.0] - 2026-05-19

### Added

- Added a Debloat section with tweaks for Microsoft Edge, Brave, Microsoft Copilot, OneDrive, and Razer software auto-install prevention.
- Added a Context Menu section with a "Create Symbolic Link" action for files and folders.
- Added Winapp2/Winappx-powered cleanup coverage for Windows, browsers, applications, development tools, gaming apps, media apps, and AppX packages.
- Added Discord Rich Presence support with a settings control.
- Added the Disable CTF input tweak.

### Changed

- Improved cleanup reporting with removed/failed states for AppX cleanup and safer handling of protected or busy cleanup targets.
- Improved startup entry hydration, presentation parsing, and UI handler stability.
- Improved backup restore and backup snapshot enrichment behavior.
- Refined UI details across cleanup, startup, sidebar, titlebar, charts, tooltips, and smooth scrolling.

### Fixed

- Fixed release formatting in CI by excluding local agent documentation files from ESLint.
- Replaced the symlink helper sidecar binary with a bundled command script copied into the Windows directory when the tweak is enabled.
- Hardened browser policy reset behavior so existing machine-wide policy values are restored instead of always deleted.
- Fixed Edge and Copilot removal state checks so success markers are only written after verification.
