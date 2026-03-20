use {
    bevy::prelude::*,
    bevy_replicon::prelude::*,
    chicken_notifications::Notify,
    chicken_protocols::{ClientAuthResponse, ServerAuthChallenge, ServerAuthResult},
    chicken_states::events::session::SetConnectingStep,
    ed25519_dalek::{Signer, SigningKey},
    rand::rngs::OsRng,
    sha2::{Digest, Sha256},
    std::{fs, io::Write},
};
// In bevy_replicon 0.38: Client empfängt Server-Nachrichten direkt als MessageReader<T> (kein FromServer-Wrapper)

pub(crate) struct ClientAuthPlugin;

impl Plugin for ClientAuthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_local_identity)
            .add_systems(
                Update,
                (on_auth_challenge_received, on_auth_result_received),
            );
    }
}

/// Lokale Ed25519-Identität des Spielers.
/// Wird beim Start geladen oder neu generiert und nie über das Netzwerk übertragen (nur der öffentliche Schlüssel).
#[derive(Resource)]
pub struct LocalIdentity {
    signing_key: SigningKey,
    /// SHA-256 Hash des öffentlichen Schlüssels als Hex-String — stabile Spieler-ID
    pub player_id: String,
}

impl LocalIdentity {
    /// Gibt den öffentlichen Schlüssel (32 Bytes) zurück.
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }
}

fn setup_local_identity(mut commands: Commands) {
    // TODO: App-Datenverzeichnis statt current_dir verwenden (z.B. dirs::data_dir())
    let path = std::env::current_dir()
        .unwrap_or_default()
        .join("identity.key");

    let signing_key = if path.exists() {
        match fs::read(&path) {
            Ok(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                SigningKey::from_bytes(&arr)
            }
            Ok(_) => {
                warn!("identity.key hat unerwartete Länge, neuer Key wird generiert");
                generate_and_save_key(&path)
            }
            Err(err) => {
                warn!("Fehler beim Lesen von identity.key: {err}, neuer Key wird generiert");
                generate_and_save_key(&path)
            }
        }
    } else {
        generate_and_save_key(&path)
    };

    let player_id = hex::encode(Sha256::digest(signing_key.verifying_key().to_bytes()));
    info!("Lokale Identität geladen: player_id={}", &player_id[..16]);

    commands.insert_resource(LocalIdentity { signing_key, player_id });
}

fn generate_and_save_key(path: &std::path::Path) -> SigningKey {
    let key = SigningKey::generate(&mut OsRng);
    match fs::File::create(path) {
        Ok(mut file) => {
            if let Err(err) = file.write_all(&key.to_bytes()) {
                warn!("identity.key konnte nicht gespeichert werden: {err}");
            } else {
                info!("Neue Identität gespeichert: {:?}", path);
            }
        }
        Err(err) => {
            warn!("identity.key konnte nicht erstellt werden unter {:?}: {err}", path);
        }
    }
    key
}

/// Empfängt den Nonce-Challenge vom Server, signiert ihn, schickt die Antwort zurück
/// und wechselt zu WaitingForAccept.
fn on_auth_challenge_received(
    mut challenge_reader: MessageReader<ServerAuthChallenge>,
    mut response_writer: MessageWriter<ClientAuthResponse>,
    mut commands: Commands,
    identity: Option<Res<LocalIdentity>>,
) {
    let Some(identity) = identity else {
        return;
    };

    for message in challenge_reader.read() {
        let signature = identity.signing_key.sign(&message.nonce).to_bytes().to_vec();
        response_writer.write(ClientAuthResponse { signature });
        info!("Auth-Challenge signiert und gesendet");
        // Authenticating → WaitingForAccept
        commands.trigger(SetConnectingStep::Next);
    }
}

/// Verarbeitet das Auth-Ergebnis vom Server.
fn on_auth_result_received(
    mut result_reader: MessageReader<ServerAuthResult>,
    mut commands: Commands,
) {
    for message in result_reader.read() {
        if message.accepted {
            info!("Auth erfolgreich: player_id={}", message.player_id);
            commands.trigger(SetConnectingStep::Next);
        } else {
            let reason = message
                .reason
                .clone()
                .unwrap_or_else(|| "Unbekannter Grund".to_string());
            error!("Auth abgelehnt: {reason}");
            commands.trigger(Notify::error(format!("Verbindung abgelehnt: {reason}")));
            commands.trigger(SetConnectingStep::Failed);
        }
    }
}
