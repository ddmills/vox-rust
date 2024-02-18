use bevy::prelude::*;

pub struct TerrainPlugin;

#[derive(Debug, Copy, Clone)]
enum Vox {
    Oob,
    Empty,
    Dirt,
    Stone,
}

impl std::fmt::Display for Vox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Vox::Oob => write!(f, "Oob"),
            Vox::Empty => write!(f, "Empty"),
            Vox::Dirt => write!(f, "Dirt"),
            Vox::Stone => write!(f, "Stone"),
        }
    }
}

const MAP_SIZE_X: i16 = 16;
const MAP_SIZE_Y: i16 = 16;
const MAP_SIZE_Z: i16 = 32;
const VOXEL_SIZE: f32 = 1.0;

pub struct VoxVec {
    x: i16,
    y: i16,
    z: i16,
}

impl VoxVec {
    fn new(x: i16, y: i16, z: i16) -> Self {
        Self { x, y, z }
    }
}

#[derive(Event)]
pub struct TerrainModifiedEvent;

#[derive(Resource)]
struct Terrain {
    voxels: [[[Vox; MAP_SIZE_Z as usize]; MAP_SIZE_Y as usize]; MAP_SIZE_X as usize],
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            voxels: [[[Vox::Empty; MAP_SIZE_Z as usize]; MAP_SIZE_Y as usize]; MAP_SIZE_X as usize],
        }
    }
}

impl Terrain {
    pub fn get_by_pos(&self, x: i16, y: i16, z: i16) -> Vox {
        if self.is_pos_oob(x, y, z) {
            return Vox::Oob;
        }

        return self.voxels[x as usize][y as usize][z as usize];
    }

    pub fn get_by_vec(&self, pos: &VoxVec) -> Vox {
        if self.is_vec_oob(pos) {
            return Vox::Oob;
        }

        return self.voxels[pos.x as usize][pos.y as usize][pos.z as usize];
    }

    pub fn is_pos_oob(&self, x: i16, y: i16, z: i16) -> bool {
        return x < 0
            || y < 0
            || z < 0
            || x >= MAP_SIZE_X as i16
            || y >= MAP_SIZE_Y as i16
            || z >= MAP_SIZE_Z as i16;
    }

    pub fn is_vec_oob(&self, pos: &VoxVec) -> bool {
        return self.is_pos_oob(pos.x, pos.y, pos.z);
    }

    pub fn get_neighbors(&self, pos: &VoxVec) -> [Vox; 9 + 8 + 9] {
        let mut n = [Vox::Empty; 9 + 8 + 9];
        let mut i = 0;

        for x in 0..=2 {
            let xp = pos.x + x - 1;
            for y in 0..=2 {
                let yp = pos.y + y - 1;
                for z in 0..=2 {
                    let zp = pos.z + z - 1;
                    if x == 1 && y == 1 && z == 1 {
                        continue;
                    }
                    n[i] = self.get_by_pos(xp, yp, zp);
                    i += 1;
                }
            }
        }

        return n;
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
                    terrain.voxels[x as usize][y as usize][z as usize] = Vox::Stone;
                } else {
                    terrain.voxels[x as usize][y as usize][z as usize] = Vox::Dirt;
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
                let d = terrain.get_by_vec(&VoxVec::new(x, y, z));
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

    let pos = VoxVec::new(0, 12, 24);
    let n = terrain.get_neighbors(&pos);

    for z in 0..MAP_SIZE_Z {
        for y in 0..MAP_SIZE_Y {
            for x in 0..MAP_SIZE_X {
                let v = terrain.get_by_pos(x, y, z);
                match v {
                    Vox::Oob => continue,
                    Vox::Empty => continue,
                    Vox::Dirt => {
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
                    }
                    Vox::Stone => {
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
                };
            }
        }
    }
}
