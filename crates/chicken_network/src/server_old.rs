// pub(crate) mod discovery;

// #[cfg(feature = "server")]
// use exitcodes::ExitCode;

// use crate::shared::PlayerNameMessage;

// use {
//     aeronet::io::{
//         connection::{Disconnect, Disconnected},
//         server::{Close, Closed, Server},
//     },
//     aeronet_io::connection::DisconnectReason,
//     aeronet_replicon::server::AeronetRepliconServer,
//     aeronet_webtransport::{
//         cert,
//         server::{
//             SessionRequest, WebTransportServer, WebTransportServerClient, WebTransportServerPlugin,
//         },
//     },
//     bevy::app::AppExit,
//     bevy::prelude::*,
//     bevy_replicon::{server::ServerSystems, shared::message::client_message::FromClient},
//     chicken_notifications::Notify,
//     chicken_settings_content::networking::NetworkingSettings,
//     chicken_states::{ServerVisibility, SetServerVisibility},
//     core::time::Duration,
//     discovery::DiscoveryServerPlugin,
// };

// pub(crate) struct ServerLogicPlugin;

// impl Plugin for ServerLogicPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_plugins((WebTransportServerPlugin, DiscoveryServerPlugin))
//             .add_systems(
//                 OnEnter(ServerVisibility::GoingPublic),
//                 on_server_going_public,
//             )
//             .add_systems(OnEnter(ServerVisibility::Public), on_server_is_running)
//             .add_systems(
//                 Update,
//                 server_is_running.run_if(in_state(ServerVisibility::Public)),
//             )
//             .add_systems(
//                 OnEnter(ServerVisibility::GoingPrivate),
//                 on_server_going_private,
//             )
//             .add_systems(
//                 PreUpdate,
//                 receive
//                     .after(ServerSystems::Receive)
//                     .run_if(in_state(ServerVisibility::Public)),
//             )
//             .add_systems(Last, on_app_exit_notify_clients);
//     }
// }

// fn receive(mut pings: MessageReader<FromClient<PlayerNameMessage>>, mut commands: Commands) {
//     for ping in pings.read() {
//         info!(
//             "received ping from client `{}` with name `{}`",
//             ping.client_id, ping.message.player_name
//         );
//         let Some(entity) = ping.client_id.entity() else {
//             error!("Received ping from invalid client ID");
//             return;
//         };
//         commands
//             .entity(entity)
//             .insert(Name::new(ping.message.player_name.clone()));
//     }
// }

// fn on_server_is_public(_: On<Add, Server>, mut next_state: ResMut<NextState<ServerVisibility>>) {
//     info!("WebTransport server is fully opened");
//     next_state.set(ServerVisibility::Public);
// }

// fn on_server_is_running(_: Commands) {
//     info!("WebTransport Server is running");
// }

// fn server_is_running(
//     mut commands: Commands,
//     server_query: Query<Entity, With<WebTransportServer>>,
// ) {
//     if server_query.is_empty() {
//         {
//             commands.trigger(SetServerVisibility {
//                 transition: ServerVisibility::GoingPrivate,
//             });
//         }
//     }
// }

// fn on_server_session_request(trigger: On<SessionRequest>, clients: Query<&ChildOf>) {
//     let client = trigger.event_target();
//     let Ok(&ChildOf(server)) = clients.get(client) else {
//         return;
//     };

//     helpers::handle_server_accept_connection(client, server, trigger);
// }

// fn on_server_client_disconnected(
//     trigger: On<Disconnected>,
//     // Optional: Query for player data if needed
// ) {
//     let client_entity = trigger.event_target();

//     match &trigger.reason {
//         DisconnectReason::ByPeer(reason) => {
//             on_server_client_graceful_disconnect(client_entity, reason);
//         }
//         DisconnectReason::ByError(err) => {
//             let err_msg = err.to_string();
//             // Simple heuristic to distinguish timeout from other errors
//             if err_msg.to_lowercase().contains("timed out") {
//                 on_server_client_timeout(client_entity, err_msg);
//             } else {
//                 on_server_client_lost(client_entity, err_msg);
//             }
//         }
//         DisconnectReason::ByUser(reason) => {
//             info!("Client {client_entity} was kicked by server: {reason}");
//         }
//     }
// }

// fn on_server_client_timeout(client: Entity, msg: String) {
//     warn!("Client {client} timed out: {msg}");
//     // TODO: Cleanup player entity, notify others, etc.
// }

// fn on_server_client_lost(client: Entity, msg: String) {
//     error!("Client {client} connection lost: {msg}");
//     // TODO: Handle sudden connection loss (e.g. keep player data for reconnect)
// }

// fn on_server_client_graceful_disconnect(client: Entity, msg: &str) {
//     info!("Client {client} left the game gracefully: {msg}");
//     // TODO: Save player state, clean up entity immediately
// }

// fn on_app_exit_notify_clients(
//     mut app_exit_events: MessageReader<AppExit>,
//     mut commands: Commands,
//     client_query: Query<Entity, With<WebTransportServerClient>>,
// ) {
//     if app_exit_events.read().next().is_some() {
//         let client_count = client_query.iter().count();
//         if client_count > 0 {
//             info!("Notifying {} clients of server shutdown", client_count);
//             for client in client_query.iter() {
//                 commands.trigger(Disconnect::new(client, "Server shutting down"));
//             }
//         }
//     }
// }
