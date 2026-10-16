# Contributing

Thanks for helping with the Valheim Rust Lab. Keep changes small, reproducible and clear about the difference between original-game evidence and custom behavior.

## Before opening a pull request

1. Read the project README and the current milestone report.
2. Check the installed Valheim build and relevant source before making compatibility claims.
3. Explain what changed, how to reproduce it, and what you actually verified.
4. Include source or small synthetic fixtures only. Never commit Valheim files, extracted assets, private reference caches, saves, recordings, local evidence folders, or compiled output.
5. Keep the original executable and user saves untouched. Native runtime checks belong in an isolated game copy and private test worlds.
6. Label approximations and avoid claiming native parity from a successful build alone.

## Local setup

The current programs target Windows. Install Rust (`rustc`) and, for managed tools, the .NET 10 SDK. Install Valheim through Steam separately. The launcher expects the repository at `steamapps/valheim-lab` next to `steamapps/common/Valheim`; do not copy game files into this repository.

Build the playable program from PowerShell:

```powershell
./Build-Playable.ps1
```

Build the investigation tools with:

```powershell
./Build-Lab.ps1
```

Some managed projects reference assemblies from the local Valheim installation and the existing Mono.Cecil location. If your setup differs, document the paths or propose a portable project configuration without checking any proprietary binaries into Git.

## Pull requests

Describe the user-visible result first, then list meaningful verification and remaining limitations. For asset or behavior claims, provide a reproducible method and concise comparison data. Do not include screenshots or captures containing private game/session data unless they have been reviewed and cleared for sharing.
