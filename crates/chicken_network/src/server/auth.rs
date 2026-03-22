use {
    aeronet::io::connection::Disconnect,
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    chicken_notifications::Notify,
    chicken_protocols::{
        ClientAuthResponse, ClientIdentityHello, ServerAuthChallenge, ServerAuthResult,
    },
    chicken_settings::SettingsLoader,
    chicken_settings_content::{BlacklistEntry, PlayerRole, PlayerRoles, ServerAccessSettings},
    ed25519_dalek::{Signature, Verifier, VerifyingKey},
    rand::{RngCore, rngs::OsRng},
    sha2::{Digest, Sha256},
    std::collections::HashMap,
};

pub(crate) struct ServerAuthPlugin;

impl Plugin for ServerAuthPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingAuths>()
            .init_resource::<PlayerRegistry>()
            .init_resource::<ServerAccessSettings>()
            .add_systems(
                Update,
                (
                    handle_client_identity_hello,
                    handle_client_auth_response,
                    enforce_blacklist_on_connected,
                )
                    .chain(),
            );
    }
}

/// Zwischenspeicher für laufende Auth-Handshakes (ClientId → offene Challenge).
#[derive(Resource, Default)]
pub struct PendingAuths(pub HashMap<ClientId, PendingAuthData>);

pub struct PendingAuthData {
    pub public_key: [u8; 32],
    pub display_name: String,
    pub steam_id: Option<u64>,
    pub nonce: [u8; 32],
    /// Passwort, das der Client bei der Verbindung angegeben hat (aus ClientIdentityHello).
    pub password: Option<String>,
}

/// Registry aller erfolgreich authentifizierten Spieler dieser Session.
#[derive(Resource, Default)]
pub struct PlayerRegistry(pub HashMap<ClientId, AuthenticatedPlayer>);

pub struct AuthenticatedPlayer {
    /// SHA-256 Hash des öffentlichen Schlüssels als Hex-String
    pub player_id: String,
    pub display_name: String,
    pub steam_id: Option<u64>,
    pub public_key: [u8; 32],
    /// Bevy-Entity des Client-Session-Objekts — wird für Kick-Befehle (Disconnect) benötigt.
    /// In bevy_replicon ist ClientId::Client(entity), daher direkt aus der ClientId ableitbar.
    pub entity: Entity,
    /// Aktuelle Rolle des Spielers auf diesem Server.
    pub role: PlayerRole,
}

/// Wird gefeuert wenn einem Spieler (neu oder geändert) eine Rolle zugewiesen wird.
#[derive(Event, Debug, Clone)]
pub struct PlayerRoleAssigned {
    pub client_id: ClientId,
    pub player_id: String,
    pub entity: Entity,
    pub role: PlayerRole,
}

/// Phase 2, Schritt 1: Client schickt seinen Public Key → Server generiert Nonce und schickt Challenge.
fn handle_client_identity_hello(
    mut hello_reader: MessageReader<FromClient<ClientIdentityHello>>,
    mut challenge_writer: MessageWriter<ToClients<ServerAuthChallenge>>,
    mut pending_auths: ResMut<PendingAuths>,
) {
    for FromClient {
        client_id, message, ..
    } in hello_reader.read()
    {
        let mut nonce = [0u8; 32];
        OsRng.fill_bytes(&mut nonce);

        pending_auths.0.insert(
            *client_id,
            PendingAuthData {
                public_key: message.public_key,
                display_name: message.display_name.clone(),
                steam_id: message.steam_id,
                nonce,
                password: message.password.clone(),
            },
        );

        challenge_writer.write(ToClients {
            mode: SendMode::Direct(*client_id),
            message: ServerAuthChallenge { nonce },
        });

        info!("Auth-Challenge gesendet an Client {:?}", client_id);
    }
}

/// Phase 2, Schritt 2: Client schickt Signatur → Server verifiziert, prüft Zugangsbedingungen und sendet Ergebnis.
fn handle_client_auth_response(
    mut commands: Commands,
    mut response_reader: MessageReader<FromClient<ClientAuthResponse>>,
    mut result_writer: MessageWriter<ToClients<ServerAuthResult>>,
    mut pending_auths: ResMut<PendingAuths>,
    mut player_registry: ResMut<PlayerRegistry>,
    mut player_roles: ResMut<PlayerRoles>,
    settings: Res<ServerAccessSettings>,
    loader: Res<SettingsLoader>,
) {
    for FromClient {
        client_id, message, ..
    } in response_reader.read()
    {
        let pending = match pending_auths.0.remove(client_id) {
            Some(p) => p,
            None => {
                warn!(
                    "ClientAuthResponse von unbekanntem Client {:?} — kein pending Auth",
                    client_id
                );
                result_writer.write(ToClients {
                    mode: SendMode::Direct(*client_id),
                    message: ServerAuthResult {
                        accepted: false,
                        player_id: String::new(),
                        reason: Some("Kein offener Auth-Handshake für diesen Client".to_string()),
                    },
                });
                continue;
            }
        };

        let verify_result = (|| -> Result<(), String> {
            let sig_bytes: [u8; 64] = message.signature.as_slice().try_into().map_err(|_| {
                format!(
                    "Signatur hat falsche Länge: {} Bytes",
                    message.signature.len()
                )
            })?;
            let vk = VerifyingKey::from_bytes(&pending.public_key)
                .map_err(|e| format!("Ungültiger Public Key: {e}"))?;
            let sig = Signature::from_bytes(&sig_bytes);
            vk.verify(&pending.nonce, &sig)
                .map_err(|e| format!("Signatur-Verifikation fehlgeschlagen: {e}"))
        })();

        match verify_result {
            Ok(()) => {
                let player_id = hex::encode(Sha256::digest(pending.public_key));

                // Doppelt-Verbindungs-Check: selbe player_id darf nur einmal verbunden sein.
                // Im Debug-Build erlaubt (z.B. hosted Server + Client auf gleicher Maschine).
                #[cfg(not(debug_assertions))]
                if player_registry.0.values().any(|p| p.player_id == player_id) {
                    warn!(
                        "Verbindung abgelehnt: player_id {} bereits verbunden",
                        &player_id[..16]
                    );
                    result_writer.write(ToClients {
                        mode: SendMode::Direct(*client_id),
                        message: ServerAuthResult {
                            accepted: false,
                            player_id: String::new(),
                            reason: Some("Bereits von einer anderen Session verbunden".to_string()),
                        },
                    });
                    continue;
                }
                #[cfg(debug_assertions)]
                if player_registry.0.values().any(|p| p.player_id == player_id) {
                    warn!(
                        "Selbe player_id {} verbindet sich erneut — im Debug-Build erlaubt",
                        &player_id[..16]
                    );
                }

                // --- Zugangskontrollen (Phase 3: Autorisierung) ---

                // 1. Kapazitäts-Check
                if settings.max_players != -1
                    && player_registry.0.len() as i32 >= settings.max_players
                {
                    warn!(
                        "Verbindung abgelehnt: Server voll ({}/{})",
                        player_registry.0.len(),
                        settings.max_players
                    );
                    result_writer.write(ToClients {
                        mode: SendMode::Direct(*client_id),
                        message: ServerAuthResult {
                            accepted: false,
                            player_id: String::new(),
                            reason: Some("Server ist voll.".to_string()),
                        },
                    });
                    continue;
                }

                // 2. Blacklist-Check
                let is_blacklisted = settings.blacklist.iter().any(|entry: &BlacklistEntry| {
                    entry.player_id == player_id
                        || (entry.steam_id.is_some() && entry.steam_id == pending.steam_id)
                });
                if is_blacklisted {
                    warn!(
                        "Verbindung abgelehnt: player_id {} ist gebannt",
                        &player_id[..16]
                    );
                    result_writer.write(ToClients {
                        mode: SendMode::Direct(*client_id),
                        message: ServerAuthResult {
                            accepted: false,
                            player_id: String::new(),
                            reason: Some("Du bist von diesem Server gesperrt.".to_string()),
                        },
                    });
                    continue;
                }

                // 3. Whitelist-Check (nur aktiv wenn nicht leer)
                if !settings.whitelist.is_empty()
                    && !settings.whitelist.contains(&pending.display_name)
                {
                    warn!(
                        "Verbindung abgelehnt: '{}' nicht auf Whitelist",
                        pending.display_name
                    );
                    result_writer.write(ToClients {
                        mode: SendMode::Direct(*client_id),
                        message: ServerAuthResult {
                            accepted: false,
                            player_id: String::new(),
                            reason: Some(
                                "Du bist nicht auf der Whitelist dieses Servers.".to_string(),
                            ),
                        },
                    });
                    continue;
                }

                // 4. Passwort-Check
                if settings.password_protected {
                    let hash_ok = match (&pending.password, &settings.password_hash) {
                        (Some(client_pw), Some(server_hash)) => {
                            let client_hash = hex::encode(Sha256::digest(client_pw.as_bytes()));
                            client_hash == *server_hash
                        }
                        _ => false,
                    };
                    if !hash_ok {
                        warn!(
                            "Verbindung abgelehnt: falsches Passwort von Client {:?}",
                            client_id
                        );
                        result_writer.write(ToClients {
                            mode: SendMode::Direct(*client_id),
                            message: ServerAuthResult {
                                accepted: false,
                                player_id: String::new(),
                                reason: Some("Falsches Server-Passwort.".to_string()),
                            },
                        });
                        continue;
                    }
                }

                // --- Rolle bestimmen ---
                let entity = client_id.entity().unwrap_or(Entity::PLACEHOLDER);
                let role = if let Some(&stored) = player_roles.roles.get(&player_id) {
                    stored
                } else if player_roles.has_no_owner() {
                    player_roles.roles.insert(player_id.clone(), PlayerRole::Owner);
                    loader.save::<PlayerRoles>(&player_roles);
                    PlayerRole::Owner
                } else {
                    PlayerRole::Player
                };

                // --- Authentifizierung erfolgreich ---
                let display_name = pending.display_name.clone();
                player_registry.0.insert(
                    *client_id,
                    AuthenticatedPlayer {
                        player_id: player_id.clone(),
                        display_name: pending.display_name,
                        steam_id: pending.steam_id,
                        public_key: pending.public_key,
                        // ClientId::Client(entity) — entity direkt aus ClientId ableitbar.
                        entity,
                        role,
                    },
                );
                commands.entity(entity).insert(role);
                info!(
                    "[auth] player_id={} display_name={:?} role={:?} entity={:?}",
                    &player_id[..16],
                    display_name,
                    role,
                    entity
                );
                commands.trigger(PlayerRoleAssigned {
                    client_id: *client_id,
                    player_id: player_id.clone(),
                    entity,
                    role,
                });
                result_writer.write(ToClients {
                    mode: SendMode::Direct(*client_id),
                    message: ServerAuthResult {
                        accepted: true,
                        player_id,
                        reason: None,
                    },
                });
            }
            Err(err) => {
                error!("Auth fehlgeschlagen für Client {:?}: {err}", client_id);
                result_writer.write(ToClients {
                    mode: SendMode::Direct(*client_id),
                    message: ServerAuthResult {
                        accepted: false,
                        player_id: String::new(),
                        reason: Some("Ungültige Signatur".to_string()),
                    },
                });
            }
        }
    }
}

/// Trennt alle aktuell verbundenen Spieler, die inzwischen auf der Blacklist stehen.
/// Wird bei jeder Änderung von `ServerAccessSettings` ausgeführt.
fn enforce_blacklist_on_connected(
    settings: Res<ServerAccessSettings>,
    player_registry: Res<PlayerRegistry>,
    mut commands: Commands,
) {
    if !settings.is_changed() {
        return;
    }
    for (_client_id, player) in &player_registry.0 {
        let is_blacklisted = settings.blacklist.iter().any(|entry: &BlacklistEntry| {
            entry.player_id == player.player_id
                || (entry.steam_id.is_some() && entry.steam_id == player.steam_id)
        });
        if is_blacklisted {
            warn!("Gebannter Spieler '{}' wird getrennt.", player.display_name);
            commands.trigger(Disconnect::new(player.entity, "Du wurdest gebannt."));
        }
    }
}
