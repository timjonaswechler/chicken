use {
    bevy::prelude::*,
    chicken_settings::SettingsAppExt,
    chicken_settings_content::{PlayerRole, PlayerRoles},
};

pub(crate) struct ServerRolesPlugin;

impl Plugin for ServerRolesPlugin {
    fn build(&self, app: &mut App) {
        app.add_settings::<PlayerRoles>();
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
