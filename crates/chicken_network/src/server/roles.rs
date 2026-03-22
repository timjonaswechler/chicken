use {
    bevy::prelude::*,
    chicken_settings::{SettingsAppExt, SettingsLoader},
    chicken_settings_content::{PlayerRole, PlayerRoles},
};

pub(crate) struct ServerRolesPlugin;

impl Plugin for ServerRolesPlugin {
    fn build(&self, app: &mut App) {
        app.add_settings::<PlayerRoles>();

        // Auf einem Hosted-Server (client + server Feature) den lokalen Spieler
        // sofort als Owner registrieren, bevor der erste Remote-Client auth durchläuft.
        #[cfg(feature = "client")]
        app.add_systems(
            OnEnter(chicken_states::states::session::ServerStatus::Running),
            assign_local_player_as_owner,
        );
    }
}

/// Nur auf hosted Servern: lokalen Spieler (LocalIdentity) als Owner in PlayerRoles eintragen.
/// Läuft einmalig wenn der Server startet — bevor irgendein Client auth durchläuft.
#[cfg(feature = "client")]
fn assign_local_player_as_owner(
    identity: Option<Res<crate::client::LocalIdentity>>,
    mut player_roles: ResMut<PlayerRoles>,
    loader: Res<SettingsLoader>,
) {
    let Some(identity) = identity else {
        warn!("[roles] Hosted server gestartet, aber keine LocalIdentity vorhanden.");
        return;
    };
    if player_roles.has_no_owner() {
        info!(
            "[roles] Hosted server: lokaler Spieler {} wird Owner",
            &identity.player_id[..16]
        );
        player_roles
            .roles
            .insert(identity.player_id.clone(), PlayerRole::Owner);
        loader.save::<PlayerRoles>(&player_roles);
    }
}

/// Mögliche Moderations-Aktionen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModAction {
    /// Spieler temporär trennen (kein Bann).
    Kick,
    /// Temporärer Bann.
    TempBan,
    /// Permanenter Bann (Blacklist-Eintrag).
    Ban,
    /// Einen Spieler auf maximal Moderator hochstufen.
    Promote,
}

/// Prüft ob eine Rolle eine bestimmte Aktion ausführen darf.
pub fn can(role: PlayerRole, action: ModAction) -> bool {
    match action {
        ModAction::Kick => role >= PlayerRole::Moderator,
        ModAction::TempBan => role >= PlayerRole::Moderator,
        ModAction::Ban => role >= PlayerRole::Admin,
        ModAction::Promote => role >= PlayerRole::Admin,
    }
}
