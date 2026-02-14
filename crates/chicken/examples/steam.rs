use bevy::prelude::*;
use chicken::{steamworks::*, *};

fn steam_system(steam_client: Res<SteamworksClient>) {
    for friend in steam_client.friends().get_friends(FriendFlags::IMMEDIATE) {
        println!(
            "Friend: {:?} - {}({:?})",
            friend.id(),
            friend.name(),
            friend.state()
        );
    }
}

fn main() {
    // Use the demo Steam AppId for SpaceWar
    // If the game wasn't launched through Steam (or Steam isn't running), request a relaunch via Steam
    // and exit this process immediately.
    #[cfg(not(debug_assertions))]
    {
        if SteamworksPlugin::restart_through_steam_if_necessary(STEAM_APP_ID) {
            return;
        }
    }

    App::new()
        // it is important to add the plugin before `RenderPlugin` that comes with `DefaultPlugins`
        .add_plugins(
            SteamworksPlugin::init_app(STEAM_APP_ID).expect("Failed to initialize Steamworks"),
        )
        .add_plugins(DefaultPlugins)
        .add_plugins(ChickenPlugin)
        .add_systems(Startup, steam_system)
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
