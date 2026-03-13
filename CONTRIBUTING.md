# Contributing

## Repos

| Repo | Zweck |
|------|-------|
| `chicken` | Rust Workspace — Game-Lib (dieses Repo) |
| `fos_client` | Bevy Game Client — konsumiert `chicken` |

## Branch-Strategie

- Entwicklung auf Feature-Branches: `feat/server-visibility`, `fix/crash-on-disconnect`
- Kein direktes Pushen auf `main`
- PRs immer via **Squash and Merge** (als einzige Merge-Option konfiguriert)
- PR-Titel = Squash-Commit = Changelog-Eintrag → sorgfältig wählen

## Commit-Konventionen

[Conventional Commits](https://www.conventionalcommits.org/) Format:

```
feat(scope): kurze Beschreibung
fix(scope): kurze Beschreibung
chore(scope): kurze Beschreibung
refactor(scope): kurze Beschreibung
test(scope): kurze Beschreibung
```

## Feature Flags

`chicken_states` erfordert immer eines der folgenden Feature-Flags:

| Flag | Zweck |
|------|-------|
| `hosted` | fos_client (Singleplayer + Multiplayer Host + Client) |
| `headless` | Dedicated Server Binary |

## Tests

```bash
just test                    # interaktiver Wizard
just test -c chicken_states  # spezifisches Crate
```

## Release

```bash
just release 0.2.0
```

1. Bumpt workspace version in root `Cargo.toml` (alle Sub-Crates erben)
2. Commit + Tag + Push
3. GitHub Actions erstellen automatisch:
   - GitHub Release mit Changelog (nur aktueller Tag)
   - PR in `fos_client` mit den Release Notes als Body

## Changelog

```bash
just changelog   # generiert CHANGELOG.md lokal (gitignored)
```

Der Changelog wird automatisch bei jedem Release via GitHub Actions generiert.
`chore(release)` Commits werden gefiltert und erscheinen nicht im Changelog.

## Workspace Version

Alle Crates erben die Version aus dem Root `Cargo.toml`:

```toml
[workspace.package]
version = "0.x.x"
```

Nie einzelne Crate-Versionen manuell ändern — immer `just release` nutzen.
