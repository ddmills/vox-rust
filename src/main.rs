use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use camera::FlyCamera;

mod camera;
mod terrain;

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins)
        .add_plugins(terrain::TerrainPlugin)
        .add_plugins(terrain::TerrainRenderPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-22.5, 0., 0.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCamera,
    ));
}
