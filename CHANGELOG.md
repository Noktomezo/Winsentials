# Changelog

## 0.4.0 - 2026-04-13

### ✨ Added

- Added two new tweak categories: **Privacy** and **Memory**.
- Added a dedicated **Privacy** page with a broader set of telemetry and data-collection controls.
- Added a dedicated **Memory** page for tweaks focused on stability and background service behavior.
- Added new privacy tweaks for:
  - disabling core Windows telemetry
  - disabling telemetry scheduled tasks
  - disabling .NET telemetry
  - disabling PowerShell telemetry
  - disabling Windows Error Reporting
  - disabling input data collection
  - disabling targeted advertising
  - disabling cloud sync
  - disabling location data collection
  - disabling Inventory Collector
- Added new memory and input tweaks, including:
  - **SvcHost Split Threshold**
  - **CSRSS High Priority**
  - wallpaper JPEG compression control for higher-quality desktop backgrounds

### 🎨 Improved

- Refined tweak cards to use a smarter, more compact layout with better width distribution for long titles.
- Improved consistency of controls across tweak cards, including switches, select menus, badges, and reset/apply actions.
- Polished dropdown menus and select behavior for better alignment, clearer state feedback, and more consistent interaction.
- Improved startup item presentation, including better handling of scheduled-task-backed entries and script-host launches.
- Expanded localization coverage for the newly added tweak categories and options.

### 🛠 Fixed

- Fixed multiple edge cases in privacy tweaks so mixed or partial states are handled more safely.
- Fixed several registry and rollback paths for new tweaks to reduce the chance of partial application.
- Fixed remaining tweak card layout issues, including row alignment, spacing, and multi-card height behavior.
- Fixed select menu checkmark alignment.
- Fixed various UI polish issues around toast styling, switches, and tweak card interactions.

### 📦 Internal

- Bumped app version to **0.4.0** in the frontend and Tauri configuration.
