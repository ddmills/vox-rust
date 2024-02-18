use bevy::prelude::*;

pub struct TerrainPlugin;

const VOX_EMPTY: u8 = 0;
const VOX_DIRT: u8 = 1;
const VOX_STONE: u8 = 2;
const MAP_SIZE_X: usize = 16;
const MAP_SIZE_Y: usize = 16;
const MAP_SIZE_Z: usize = 32;

const VOXEL_SIZE: f32 = 1.0;

#[derive(Event)]
pub struct TerrainModifiedEvent;

#[derive(Resource)]
struct Terrain {
    voxels: [[[u8; MAP_SIZE_Z]; MAP_SIZE_Y]; MAP_SIZE_X],
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            voxels: [[[VOX_EMPTY; MAP_SIZE_Z]; MAP_SIZE_Y]; MAP_SIZE_X],
        }
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Terrain>()
            .add_event::<TerrainModifiedEvent>()
            .add_systems(Startup, (generate_voxels, debug_voxels).chain());
    }
}

fn generate_voxels(
    mut terrain: ResMut<Terrain>,
    mut ev_terrain_mod: EventWriter<TerrainModifiedEvent>,
) {
    for z in 16..MAP_SIZE_Z {
        for y in 0..MAP_SIZE_Y {
            for x in 0..MAP_SIZE_X {
                if z > 24 {
                    terrain.voxels[x][y][z] = VOX_STONE;
                } else {
                    terrain.voxels[x][y][z] = VOX_DIRT;
                }
            }
        }
    }

    ev_terrain_mod.send(TerrainModifiedEvent {});
}

fn debug_voxels(terrain: Res<Terrain>) {
    for z in 0..MAP_SIZE_Z {
        println!("z={}", z);
        for y in 0..MAP_SIZE_Y {
            for x in 0..MAP_SIZE_X {
                let d = terrain.voxels[x][y][z];
                print!("{}", d);
            }
            println!("");
        }
    }
}

pub struct TerrainRenderPlugin;

impl Plugin for TerrainRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_voxels);
    }
}

fn render_voxels(
    terrain: Res<Terrain>,
    mut ev_terrain_mod: EventReader<TerrainModifiedEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if ev_terrain_mod.is_empty() {
        return;
    }

    ev_terrain_mod.clear();
    let cube = meshes.add(Mesh::from(shape::Cube { size: 0.5 }));
    let dirt = materials.add(Color::rgb_u8(255, 144, 100).into());
    let stone = materials.add(Color::rgb_u8(124, 124, 124).into());

    for z in 0..MAP_SIZE_Z {
        for y in 0..MAP_SIZE_Y {
            for x in 0..MAP_SIZE_X {
                let v = terrain.voxels[x][y][z];
                if v == VOX_EMPTY {
                    continue;
                } else if v == VOX_DIRT {
                    commands.spawn(PbrBundle {
                        mesh: cube.clone(),
                        material: dirt.clone(),
                        transform: Transform::from_xyz(
                            x as f32 * VOXEL_SIZE,
                            (MAP_SIZE_Z - z) as f32 * VOXEL_SIZE,
                            y as f32 * VOXEL_SIZE,
                        ),
                        ..default()
                    });
                } else if v == VOX_STONE {
                    commands.spawn(PbrBundle {
                        mesh: cube.clone(),
                        material: stone.clone(),
                        transform: Transform::from_xyz(
                            x as f32 * VOXEL_SIZE,
                            (MAP_SIZE_Z - z) as f32 * -VOXEL_SIZE,
                            y as f32 * VOXEL_SIZE,
                        ),
                        ..default()
                    });
                }
            }
        }
    }
}
