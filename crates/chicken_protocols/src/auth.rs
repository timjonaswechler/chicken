#[cfg(any(feature = "hosted", feature = "headless"))]
use {bevy::prelude::*, bevy_replicon::prelude::*, serde::{Deserialize, Serialize}};

// ─── Client → Server ─────────────────────────────────────────────────────────

/// Erste Nachricht des Clients nach dem QUIC-Handshake (ConnectingStep::Authenticating).
/// Enthält den öffentlichen Ed25519-Schlüssel für die Challenge-Response-Authentifizierung.
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientIdentityHello {
    /// Ed25519 Verifying Key (öffentlicher Schlüssel, 32 Bytes)
    pub public_key: [u8; 32],
    /// Anzeigename des Spielers
    pub display_name: String,
    /// Optional: Steam-ID falls Spieler über Steam verbunden
    pub steam_id: Option<u64>,
}

/// Signatur-Antwort des Clients auf den Server-Challenge.
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ClientAuthResponse {
    /// Ed25519-Signatur über den vom Server gesendeten Nonce (immer 64 Bytes)
    pub signature: Vec<u8>,
}

// ─── Server → Client ─────────────────────────────────────────────────────────

/// Server sendet einen zufälligen Nonce als Challenge an den Client.
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerAuthChallenge {
    /// Zufällige 32 Bytes die der Client signieren muss
    pub nonce: [u8; 32],
}

/// Endgültiges Authentifizierungsergebnis vom Server.
#[cfg(any(feature = "hosted", feature = "headless"))]
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct ServerAuthResult {
    /// true = Authentifizierung erfolgreich, false = abgelehnt
    pub accepted: bool,
    /// SHA-256 Hash des öffentlichen Schlüssels als Hex-String (stabile Spieler-ID)
    pub player_id: String,
    /// Ablehnungsgrund (nur bei accepted = false gesetzt)
    pub reason: Option<String>,
}
