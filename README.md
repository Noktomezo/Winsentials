<p align="center">
  <img src="assets/app-thumbnail.png" alt="Winsentials thumbnail" />
</p>

<p align="center">
  ❤️‍🔥 <strong>Winsentials</strong> is a desktop application for Windows 10 and 11 that lets you tune performance, privacy, appearance, and system behavior through a clean interface built on top of low-level system settings. ⛓️‍💥
</p>

<p align="center">
  <a href="https://github.com/Noktomezo/Winsentials/releases/latest"><img alt="Version" src="https://img.shields.io/github/v/release/Noktomezo/Winsentials?style=flat&label=Version&logo=github&logoColor=white&color=0f766e&labelColor=134e4a" /></a>
  <a href="https://github.com/Noktomezo/Winsentials/stargazers"><img alt="Stars" src="https://img.shields.io/github/stars/Noktomezo/Winsentials?style=flat&label=Stars&logo=github&logoColor=white&color=0f766e&labelColor=134e4a" /></a>
  <a href="https://github.com/Noktomezo/Winsentials/releases"><img alt="Downloads" src="https://img.shields.io/github/downloads/Noktomezo/Winsentials/total?style=flat&label=Downloads&logo=github&logoColor=white&color=0f766e&labelColor=134e4a" /></a>
  <a href="https://github.com/Noktomezo/Winsentials/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/github/license/Noktomezo/Winsentials?style=flat&label=License&logo=github&logoColor=white&color=0f766e&labelColor=134e4a" /></a>
</p>

## 🥶 Disclaimer

Winsentials is currently under active development and is not recommended for use right now.

## Local Zed + vexp setup

If you use Zed with the local `vexp` MCP server:

1. Copy `.zed/settings.json.example` to `.zed/settings.json`
2. Update `context_servers.vexp.command.path` to point to your local `vexp-core.exe`
3. Keep `args` set to `["mcp"]`

The real `.zed/settings.json` is ignored on purpose so each contributor can keep a machine-specific path locally.
