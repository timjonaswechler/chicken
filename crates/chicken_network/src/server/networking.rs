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
    // pub mod helpers {
    //     use {
    //         aeronet_webtransport::server::{SessionRequest, SessionResponse},
    //         bevy::prelude::*,
    //     };

    //     pub(super) fn handle_server_accept_connection(
    //         client: Entity,
    //         server: Entity,
    //         mut trigger: On<SessionRequest>,
    //     ) {
    //         info!("{client} connecting to {server} with headers:");
    //         for (header_key, header_value) in &trigger.headers {
    //             info!("  {header_key}: {header_value}");
    //         }

    //         trigger.respond(SessionResponse::Accepted);
    //     }

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
