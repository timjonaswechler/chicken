# Claude Instructions

Lies zuerst [`CONTRIBUTING.md`](../CONTRIBUTING.md) — dort sind Branch-Strategie, Commit-Konventionen, Feature-Flags, Tests und Release-Prozess dokumentiert.

## Crate-Struktur

| Crate | Zweck |
|-------|-------|
| `chicken` | Haupt-Plugin, aggregiert alle Sub-Crates |
| `chicken_states` | State Machines (AppScope, SessionType, Server/Client flows) |
| `chicken_protocols` | Netzwerk-Protokolle (Chat, Replicon Messages) |
| `chicken_network` | Aeronet / WebTransport Netzwerk-Layer |
| `chicken_identity` | PlayerIdentity Resource + IdentityChanged Event |
| `chicken_notifications` | In-Game Notification System |
| `chicken_settings` | Settings-System |

## Feature-Flag Kette

```
App (campfire)
  → chicken/hosted
    → chicken_states/hosted
    → chicken_protocols/hosted
    → chicken_network/client
```

`hosted` = Singleplayer + Multiplayer Host + Client (mit UI)
`headless` = Dedicated Server (kein UI)

`chicken_states` und `chicken_protocols` erzwingen eines der beiden Flags via `compile_error!`.

## Tests

```bash
just test --ci       # alle Crates, alle Features
just test            # interaktiver Wizard
```

Unit-Tests liegen in `src/**/*.rs` (`#[cfg(test)]`), Integration-Tests in `tests/`.
Jede Crate wird direkt getestet — `chicken` ist kein Test-Umbrella.

## Wichtige Konventionen

- **Nie direkt auf `main` committen** — immer Feature-Branch + PR
- PRs via **Squash and Merge** — PR-Titel wird Changelog-Eintrag
- `compile_error!` Guards in Crates die Feature-Flags erfordern (siehe `chicken_states/src/lib.rs`)
- Bevy State-Transitions sind deferred (wirken erst nächsten Frame) → kein Multi-Transition pro Frame
