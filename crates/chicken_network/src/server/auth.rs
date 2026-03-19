use {
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    chicken_protocols::{ClientAuthResponse, ClientIdentityHello, ServerAuthChallenge, ServerAuthResult},
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
            .add_systems(
                Update,
                (handle_client_identity_hello, handle_client_auth_response).chain(),
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
}

/// Phase 2, Schritt 1: Client schickt seinen Public Key → Server generiert Nonce und schickt Challenge.
fn handle_client_identity_hello(
    mut hello_reader: MessageReader<FromClient<ClientIdentityHello>>,
    mut challenge_writer: MessageWriter<ToClients<ServerAuthChallenge>>,
    mut pending_auths: ResMut<PendingAuths>,
) {
    for FromClient { client_id, message, .. } in hello_reader.read() {
        let mut nonce = [0u8; 32];
        OsRng.fill_bytes(&mut nonce);

        pending_auths.0.insert(
            *client_id,
            PendingAuthData {
                public_key: message.public_key,
                display_name: message.display_name.clone(),
                steam_id: message.steam_id,
                nonce,
            },
        );

        challenge_writer.write(ToClients {
            mode: SendMode::Direct(*client_id),
            message: ServerAuthChallenge { nonce },
        });

        info!("Auth-Challenge gesendet an Client {:?}", client_id);
    }
}

/// Phase 2, Schritt 2: Client schickt Signatur → Server verifiziert und sendet Ergebnis.
fn handle_client_auth_response(
    mut response_reader: MessageReader<FromClient<ClientAuthResponse>>,
    mut result_writer: MessageWriter<ToClients<ServerAuthResult>>,
    mut pending_auths: ResMut<PendingAuths>,
    mut player_registry: ResMut<PlayerRegistry>,
) {
    for FromClient { client_id, message, .. } in response_reader.read() {
        let pending = match pending_auths.0.remove(client_id) {
            Some(p) => p,
            None => {
                warn!("ClientAuthResponse von unbekanntem Client {:?} — kein pending Auth", client_id);
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
            let sig_bytes: [u8; 64] = message
                .signature
                .as_slice()
                .try_into()
                .map_err(|_| format!("Signatur hat falsche Länge: {} Bytes", message.signature.len()))?;
            let vk = VerifyingKey::from_bytes(&pending.public_key)
                .map_err(|e| format!("Ungültiger Public Key: {e}"))?;
            let sig = Signature::from_bytes(&sig_bytes);
            vk.verify(&pending.nonce, &sig)
                .map_err(|e| format!("Signatur-Verifikation fehlgeschlagen: {e}"))
        })();

        match verify_result {
            Ok(()) => {
                let player_id = hex::encode(Sha256::digest(pending.public_key));
                player_registry.0.insert(
                    *client_id,
                    AuthenticatedPlayer {
                        player_id: player_id.clone(),
                        display_name: pending.display_name,
                        steam_id: pending.steam_id,
                        public_key: pending.public_key,
                    },
                );
                info!("Client {:?} authentifiziert: player_id={}", client_id, &player_id[..16]);
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
