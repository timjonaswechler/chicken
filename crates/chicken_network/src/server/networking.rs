pub mod ports {
    // https://en.wikipedia.org/wiki/List_of_TCP_and_UDP_port_numbers#Registered_ports
    use std::net::UdpSocket;

    #[allow(dead_code)]
    pub(crate) fn is_port_available(port: u16) -> bool {
        if UdpSocket::bind(("0.0.0.0", port)).is_err() {
            return false;
        }
        true
    }

    pub(in crate::server) fn find_free_port(start_port: u16, max_attempts: usize) -> Option<u16> {
        (0..max_attempts).find_map(|i| {
            let port = start_port + i as u16;
            if port > u16::MAX {
                return None;
            }
            if is_port_available(port) {
                Some(port)
            } else {
                None
            }
        })
    }
}

pub mod address {
    pub mod helpers {
        use {
            aeronet_webtransport::server::{SessionRequest, SessionResponse},
            bevy::prelude::*,
        };

        pub fn accept_session_request(mut trigger: On<SessionRequest>) {
            // TODO: hier später Blacklist/Whitelist-Prüfung, Passwort, Server-voll-Check
            //   Phase 1 — Transport (Aeronet/QUIC/TLS)
            //   "Kann ich überhaupt sicher mit diesem Server reden?"
            //   → Verschlüsselter Tunnel steht
            //   → Niemand kann mitlesen oder manipulieren
            //   → Noch keine Ahnung wer der Spieler ist

            // Phase 2 — Identität (chicken_protocols, Ed25519)
            //   "Wer bist du, und bist du wirklich der?"
            //   → Public Key + Challenge-Response
            //   → Server weiß jetzt: das ist Spieler-ID abc123
            //   → Identität kryptographisch bewiesen

            // Phase 3 — Autorisierung (chicken_network, Serverlogik)
            //   "Darfst du rein?"
            //   → Gebannt? → Kick
            //   → Server voll? → Reject
            //   → Passwort falsch? → Reject
            //   → Whitelist-only und nicht drauf? → Reject
            //   → Alles okay → Welcome
            //
            // Der konkrete Flow
            // 1. QUIC Handshake (Aeronet, TLS 1.3)       ← bleibt wie es ist
            //         ↓  Verbindung steht, verschlüsselt
            // 2. ConnectingStep::Authenticating           ← hier kommt die neue Logik
            //    Client → Server: { public_key, display_name }
            //    Server → Client: { nonce: random_bytes }
            //    Client → Server: { signature: sign(nonce, private_key) }
            //    Server:           verify(signature, nonce, public_key) → Accept/Reject
            //         ↓  Spieler authentifiziert
            // 3. ConnectingStep::WaitingForAccept         ← wartet auf Accept-Nachricht
            //         ↓
            // 4. Sync, Playing …
            trigger.respond(SessionResponse::Accepted);
        }
    }

    /// Get the local IP address of the server.
    pub fn get_local_ip() -> Option<std::net::IpAddr> {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;
        socket.local_addr().ok().map(|addr| addr.ip())
    }

    //     pub(super) fn _handle_server_reject_connection() {
    //         // TODO: client UUID or Name is on the server's blacklist
    //         // TODO: Server password is incorrect
    //         // TODO: Server is full
    //         todo!("Implement on_server_shutdown_notify_clients")
    //     }

    //     pub(crate) mod ports {
    //         // https://en.wikipedia.org/wiki/List_of_TCP_and_UDP_port_numbers#Registered_ports
    //         use std::net::UdpSocket;

    //         #[allow(dead_code)]
    //         pub(crate) fn is_port_available(port: u16) -> bool {
    //             if UdpSocket::bind(("0.0.0.0", port)).is_err() {
    //                 return false;
    //             }
    //             true
    //         }

    //         pub(in crate::server_old) fn find_free_port(
    //             start_port: u16,
    //             max_attempts: usize,
    //         ) -> Option<u16> {
    //             (0..max_attempts).find_map(|i| {
    //                 let port = start_port + i as u16;
    //                 if port > u16::MAX {
    //                     return None;
    //                 }
    //                 if is_port_available(port) {
    //                     Some(port)
    //                 } else {
    //                     None
    //                 }
    //             })
    //         }
    //     }
    // }
}
