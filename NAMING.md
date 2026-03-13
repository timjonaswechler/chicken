# Naming Conventions — Chicken105

## Regel

| Kategorie | Universum | Beispiele |
|-----------|-----------|-----------|
| Interne Library-Crates | `chicken_*` | `chicken_states`, `chicken_network` |
| Studio-weite Tools | Hühnerhof — playful | `scratch`, `brood`, `roost` |
| Forge of Stories | Fantasy / DnD | `campfire`, `bastion`, `oracle` |
| The Last Alchemist | Alchemie & mittelalterliche Mystik | `athanor`, `retort`, `azoth` |

---

## Interne Library-Crates

Bleiben immer `chicken_*`. Werden nie umbenannt.

```
chicken_states
chicken_network
chicken_protocols
chicken_settings
chicken_identity
...
```

---

## Studio-weit — Hühnerhof

Für Tools und Infrastruktur die nicht spielspezifisch sind.

| Name | Bedeutung | Status |
|------|-----------|--------|
| `scratch` | Scharren | Dev CLI |
| `brood` | Brut | Test Runner |
| `roost` | Hühnerstange | Hosting / Deployment |
| `molt` | Mauser | Update / Migration |
| `yolk` | Eigelb | Shared Config |
| `clutch` | Gelege | Release / Artifact Bundle |
| `peck` | Picken | kleines CLI-Tool |

---

## Forge of Stories — Fantasy / DnD

Bodenständig-abenteuerlich. Leicht chaotisch, lebendig — Dwarf Fortress Vibe.

| Name | Bedeutung | Status |
|------|-----------|--------|
| `campfire` | Lagerfeuer, wo Abenteuer beginnen | ✅ Game Client |
| `bastion` | uneinnehmbare Festung | ✅ Dedicated Server |
| `oracle` | Orakel | Monitoring / Dashboard |
| `wizard` | orchestriert Dinge im Hintergrund | Dev CLI |
| `herald` | Bote | Discovery / Server Browser |
| `grimoire` | Zauberbuch | Config / Settings Tool |
| `scribe` | Schreiber | Logging / Replay |
| `cartographer` | Kartograf | World Editor |
| `dungeon` | Verlies | Headless Test-Server |
| `saga` | aufgezeichnete Geschichte | Replay / Chronicle |
| `ember` | was nach dem Chaos übrig bleibt | Crash Reporter |

---

## The Last Alchemist — Alchemie & mittelalterliche Mystik

Poetisch, geheimnisvoll, historisch.

| Name | Bedeutung | Status |
|------|-----------|--------|
| `athanor` | alchemistischer Dauerofen | Game Client (Kandidat) |
| `retort` | Destillationsgefäß | Dedicated Server (Kandidat) |
| `alembic` | Destillierhelm | Dev Tool |
| `azoth` | universelles Lösungsmittel | CLI Tool |
| `aether` | 5. Element, verbindet alles | Networking Layer |
| `vitriol` | Grüner Vitriol | Crash / Error Reporter |
| `materia` | Rohmaterial | Asset Pipeline |
| `homunculus` | künstlicher Mensch | Bot / NPC Simulator |
| `phylactery` | Behältnis der Seele | Save / Persistence |
| `tincture` | Tinktur | Patch / Hotfix Tool |
| `calcine` | Rösten zu Asche | Build / Cleanup Tool |
