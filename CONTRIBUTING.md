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

## CI

Bei jedem Push und PR auf `main` läuft automatisch:
1. `cargo xtask test --ci` — alle Crates mit allen Features + Integration Tests
2. Code Coverage Report — sichtbar im GitHub Actions Summary Tab

Crates mit pre-existing Compile-Fehlern sind via `ci: false` in `xtask/src/config.rs` ausgeschlossen (siehe Issue #12).

## Release

```bash
just release 0.2.0
```

1. Bumpt workspace version in root `Cargo.toml` (alle Sub-Crates erben)
2. Commit + Tag + Push
3. GitHub Actions (`release.yml`) läuft automatisch:
   - Tests + Coverage
   - GitHub Release mit Changelog (nur aktueller Tag) + Coverage-Tabelle
   - PR in `fos_client` mit chicken Release Notes als Body

## fos_client Update (nach chicken Release)

1. Automatischer PR landet in `fos_client` mit chicken Release Notes
2. PR-Titel anpassen: "was merkt der User?" → wird der fos_client Changelog-Eintrag
3. Squash and Merge

## Changelog

```bash
just changelog   # generiert CHANGELOG.md lokal (gitignored)
```

Wird automatisch bei jedem Release via GitHub Actions generiert.
`chore(release)` Commits werden gefiltert und erscheinen nicht im Changelog.

## Workspace Version

Alle Crates erben die Version aus dem Root `Cargo.toml`:

```toml
[workspace.package]
version = "0.x.x"
```

Nie einzelne Crate-Versionen manuell ändern — immer `just release` nutzen.
