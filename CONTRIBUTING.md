# Contributing

## Repos

| Repo | Zweck | Pfad | 
|------|-------|----|
| `chicken` | Rust Workspace — Game-Lib (dieses Repo) | `.` | 
| `campfire` | Bevy Game Client — konsumiert `chicken` | `../fos/campfire` |

## Branch-Strategie

- Entwicklung auf Feature-Branches — **nie direkt auf `main` committen**
- PRs immer via **Squash and Merge** (als einzige Merge-Option konfiguriert)
- PR-Titel = Squash-Commit = Changelog-Eintrag → sorgfältig wählen

### Branch-Naming

```
feat/<scope>-<issue-nr>       feat/server-visibility-9
fix/<scope>-<issue-nr>        fix/chat-e2e-31
refactor/<scope>-<issue-nr>   refactor/protocols-feature-flags-26
chore/<scope>-<issue-nr>      chore/states-compile-error-11
```

Prefix entspricht dem Conventional-Commit-Typ. `<scope>` ist das betroffene Subsystem, `<issue-nr>` die zugehörige Issue-Nummer.

## Commit-Konventionen

[Conventional Commits](https://www.conventionalcommits.org/) Format:

```
feat(scope): kurze Beschreibung
fix(scope): kurze Beschreibung
chore(scope): kurze Beschreibung
refactor(scope): kurze Beschreibung
test(scope): kurze Beschreibung
```

## Labels

Labels werden sowohl für Issues als auch für PRs genutzt — immer **Typ + Bereich** kombinieren.

### Typ
| Label | Bedeutung |
|-------|-----------|
| `bug` | Etwas funktioniert nicht |
| `enhancement` | Neues Feature oder Verbesserung |
| `documentation` | Dokumentation |
| `meta` | Projekt-Infrastruktur, CI, Tooling |

### Qualität
| Label | Bedeutung |
|-------|-----------|
| `code-quality` | Code lesbarer/sauberer machen |
| `architectural` | Strukturelle/Pattern-Änderungen |
| `dev-ex` | Developer Experience verbessern |
| `performance` | Laufzeit-Optimierungen |
| `controversial` | Braucht sorgfältige Analyse vor Umsetzung |

### Status
| Label | Bedeutung |
|-------|-----------|
| `blocked` | Wartet auf anderes Issue oder externes |
| `good first issue` | Klarer Scope, guter Einstieg |
| `critical` | Crash oder komplett kaputtes Core-Feature |
| `panic` | `unwrap`/`expect`/`todo!` die zur Laufzeit paniken |

### Bereich
`states` · `protocols` · `identity` · `settings` · `ui` · `chat` · `steam` · `notifications`

## Feature Flags

`chicken_states` erfordert immer eines der folgenden Feature-Flags:

| Flag | Zweck |
|------|-------|
| `hosted` | campfire (Singleplayer + Multiplayer Host + Client) |
| `headless` | Dedicated Server Binary |

## Tests

```bash
just test                    # interaktiver Wizard
just test -c chicken_states  # spezifisches Crate
```

## CI

Bei jedem Push und PR auf `main` läuft automatisch:
1. `just test --ci` — alle Crates mit allen Features + Integration Tests
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
   - PR in `campfire` mit chicken Release Notes als Body

## campfire Update (nach chicken Release)

1. Automatischer PR landet in `campfire` mit chicken Release Notes
2. PR-Titel anpassen: "was merkt der User?" → wird der campfire Changelog-Eintrag
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
