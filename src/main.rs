use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    gizmos,
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
};
use camera::FlyCamera;
use slice::SlicePlugin;
use terrain::TerrainMaterial;

mod camera;
mod slice;
mod terrain;

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_plugins((DefaultPlugins, MaterialPlugin::<TerrainMaterial>::default()))
        .add_plugins(terrain::TerrainPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(SlicePlugin)
        .add_plugins(WireframePlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, draw_gizmos)
        .run();
}

fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::X * 100., Color::RED);
    gizmos.line(Vec3::ZERO, Vec3::Y * 100., Color::GREEN);
    gizmos.line(Vec3::ZERO, Vec3::Z * 100., Color::BLUE);
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

    let cube = meshes.add(Cuboid::new(0.75, 0.75, 0.75));
    let stone = materials.add(Color::rgb_u8(124, 124, 124));

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
            transform: Transform::from_xyz(-10., 0., -10.)
                .looking_at(Vec3::new(5., 10., 10.), Vec3::Y),
            ..default()
        },
        FlyCamera,
    ));
}
