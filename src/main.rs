use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::wireframe::{Wireframe, WireframePlugin},
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
        .add_plugins(WireframePlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });

    let cube = meshes.add(Mesh::from(shape::Cube { size: 0.75 }));
    let stone = materials.add(Color::rgb_u8(124, 124, 124).into());

    commands.spawn((
        PbrBundle {
            mesh: cube.clone(),
            material: stone.clone(),
            ..default()
        },
        Wireframe,
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-10., 0., -10.).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FlyCamera,
    ));
}
