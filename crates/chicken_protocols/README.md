# chicken_protocols

Pure protocol definition crate — message types, channel configuration, and Bevy message registration.

## Responsibility

This crate defines **what messages exist** and **how they are registered** with `bevy_replicon`. It does not contain any game logic, server systems, or client systems.

| What belongs here | What does NOT belong here |
|---|---|
| Message structs (`ClientChat`, `ServerChat`, …) | Server-side handlers (`handle_client_chat`, …) |
| Channel registration (`.add_client_message`, …) | Resources with server state (`ChatHistory`, …) |
| Protocol constants (`CHAT_MESSAGE_MAX_LENGTH`, …) | System scheduling / `add_systems` |
| Pure parsing utilities (`extract_command`, …) | Client-side receive logic |

## Structure

- `src/auth.rs` — Auth protocol messages (Ed25519 challenge-response)
- `src/chat.rs` — Chat protocol messages + parsing utilities

## Feature flags

Requires exactly one of `hosted` or `headless` — enforced via `compile_error!` in `lib.rs`.
