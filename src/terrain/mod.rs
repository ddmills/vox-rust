use bevy::{
    pbr::wireframe::Wireframe,
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute},
        render_resource::PrimitiveTopology,
    },
    utils::petgraph::adj::Neighbors,
};

pub struct TerrainPlugin;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Block {
    Oob,
    Empty,
    Dirt,
    Stone,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum BlockVisibility {
    Empty,
    Opaque,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Block::Oob => write!(f, "Oob"),
            Block::Empty => write!(f, "Empty"),
            Block::Dirt => write!(f, "Dirt"),
            Block::Stone => write!(f, "Stone"),
        }
    }
}

impl Block {
    pub fn is_filled(&self) -> bool {
        match *self {
            Block::Oob => false,
            Block::Empty => false,
            Block::Dirt => true,
            Block::Stone => true,
        }
    }

    pub fn visibility(&self) -> BlockVisibility {
        match *self {
            Block::Oob => BlockVisibility::Empty,
            Block::Empty => BlockVisibility::Empty,
            Block::Dirt => BlockVisibility::Opaque,
            Block::Stone => BlockVisibility::Opaque,
        }
    }
}

const MAP_SIZE_X: i16 = 16;
const MAP_SIZE_Z: i16 = 16;
const MAP_SIZE_Y: i16 = 32;
const BLOCK_SIZE: f32 = 1.;

#[derive(Event)]
pub struct TerrainModifiedEvent;

#[derive(Resource)]
struct Terrain {
    blocks: [[[Block; MAP_SIZE_Y as usize]; MAP_SIZE_Z as usize]; MAP_SIZE_X as usize],
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            blocks: [[[Block::Empty; MAP_SIZE_Y as usize]; MAP_SIZE_Z as usize];
                MAP_SIZE_X as usize],
        }
    }
}

impl Terrain {
    pub fn get(&self, x: i16, y: i16, z: i16) -> Block {
        if self.is_pos_oob(x, y, z) {
            return Block::Oob;
        }

        return self.blocks[x as usize][z as usize][y as usize];
    }

    pub fn is_pos_oob(&self, x: i16, y: i16, z: i16) -> bool {
        return x < 0
            || y < 0
            || z < 0
            || x >= MAP_SIZE_X as i16
            || y >= MAP_SIZE_Y as i16
            || z >= MAP_SIZE_Z as i16;
    }

    pub fn get_neighbors_immediate(&self, x: i16, y: i16, z: i16) -> [Block; 6] {
        [
            self.get(x, y + 1, z), // above
            self.get(x, y, z - 1), // front
            self.get(x + 1, y, z), // right
            self.get(x, y, z + 1), // behind
            self.get(x - 1, y, z), // left
            self.get(x, y - 1, z), // below
        ]
    }

    pub fn get_neighbors(&self, x: i16, y: i16, z: i16) -> [Block; 9 + 8 + 9] {
        let mut n = [Block::Empty; 9 + 8 + 9];
        let mut i = 0;

        for dx in 0..=2 {
            let cx = x + dx - 1;
            for dz in 0..=2 {
                let cz = z + dz - 1;
                for dy in 0..=2 {
                    let cy = y + dy - 1;
                    if dx == 1 && dy == 1 && dz == 1 {
                        continue;
                    }
                    n[i] = self.get(cx, cy, cz);
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
            .add_systems(Startup, (generate_voxels, debug_blocks).chain());
    }
}

fn generate_voxels(
    mut terrain: ResMut<Terrain>,
    mut ev_terrain_mod: EventWriter<TerrainModifiedEvent>,
) {
    let rad = MAP_SIZE_X as f32 / 2.;
    let center = Vec3::new(
        MAP_SIZE_X as f32 / 2.,
        MAP_SIZE_Y as f32 / 2.,
        MAP_SIZE_Z as f32 / 2.,
    );
    for x in 0..MAP_SIZE_X {
        for z in 0..MAP_SIZE_Z {
            for y in 0..24 {
                let pos = Vec3::new(x as f32, y as f32, z as f32);

                if pos.distance(center) < rad {
                    if y < 16 {
                        terrain.blocks[x as usize][z as usize][y as usize] = Block::Stone;
                    } else {
                        terrain.blocks[x as usize][z as usize][y as usize] = Block::Dirt;
                    }
                }
            }
        }
    }

    ev_terrain_mod.send(TerrainModifiedEvent {});
}

fn debug_blocks(terrain: Res<Terrain>) {
    for y in 0..MAP_SIZE_Y {
        println!("y={}", y);
        for z in 0..MAP_SIZE_Z {
            for x in 0..MAP_SIZE_X {
                let d = terrain.get(x, y, z);
                print!("{}", d);
            }
            println!("");
        }
    }
}

pub struct TerrainRenderPlugin;

impl Plugin for TerrainRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_blocks);
    }
}

fn render_blocks(
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
    let mesh = meshes.add(mesh_terrain(&terrain));
    let dirt = materials.add(Color::rgb_u8(255, 144, 100).into());

    commands.spawn((
        PbrBundle {
            mesh: mesh.clone(),
            material: dirt.clone(),
            ..default()
        },
        // Wireframe,
    ));
}

#[derive(Default)]
struct TerrainMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indicies: Vec<u32>,
}

fn mesh_terrain(terrain: &Res<Terrain>) -> Mesh {
    let mut data = TerrainMeshData::default();
    data.positions = vec![];
    data.normals = vec![];
    data.indicies = vec![];

    let mut idx = 0;

    for x in 0..MAP_SIZE_X {
        for z in 0..MAP_SIZE_Z {
            for y in 0..MAP_SIZE_Y {
                let block = terrain.get(x, y, z);

                if !block.is_filled() {
                    continue;
                }

                let fx = x as f32;
                let fy = y as f32;
                let fz = z as f32;

                let neighbors = terrain.get_neighbors_immediate(x, y, z);

                if !neighbors[0].is_filled() {
                    // add face above
                    data.positions.push([fx, fy + 1., fz]);
                    data.positions.push([fx + 1., fy + 1., fz]);
                    data.positions.push([fx + 1., fy + 1., fz + 1.]);
                    data.positions.push([fx, fy + 1., fz + 1.]);

                    data.normals.push([0., 1., 0.]);
                    data.normals.push([0., 1., 0.]);
                    data.normals.push([0., 1., 0.]);
                    data.normals.push([0., 1., 0.]);

                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 0);
                    data.indicies.push(idx + 0);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);

                    idx = idx + 4;
                }

                if !neighbors[1].is_filled() {
                    // add face in front
                    data.positions.push([fx, fy, fz]);
                    data.positions.push([fx, fy + 1., fz]);
                    data.positions.push([fx + 1., fy + 1., fz]);
                    data.positions.push([fx + 1., fy, fz]);

                    data.normals.push([0., 0., -1.]);
                    data.normals.push([0., 0., -1.]);
                    data.normals.push([0., 0., -1.]);
                    data.normals.push([0., 0., -1.]);

                    data.indicies.push(idx + 0);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 0);

                    idx = idx + 4;
                }

                if !neighbors[2].is_filled() {
                    // add face right
                    data.positions.push([fx + 1., fy, fz]);
                    data.positions.push([fx + 1., fy, fz + 1.]);
                    data.positions.push([fx + 1., fy + 1., fz + 1.]);
                    data.positions.push([fx + 1., fy + 1., fz]);

                    data.normals.push([1., 0., 0.]);
                    data.normals.push([1., 0., 0.]);
                    data.normals.push([1., 0., 0.]);
                    data.normals.push([1., 0., 0.]);

                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 0);
                    data.indicies.push(idx + 0);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);

                    idx = idx + 4;
                }

                if !neighbors[3].is_filled() {
                    // add face behind
                    data.positions.push([fx, fy, fz + 1.]);
                    data.positions.push([fx, fy + 1., fz + 1.]);
                    data.positions.push([fx + 1., fy + 1., fz + 1.]);
                    data.positions.push([fx + 1., fy, fz + 1.]);

                    data.normals.push([0., 0., 1.]);
                    data.normals.push([0., 0., 1.]);
                    data.normals.push([0., 0., 1.]);
                    data.normals.push([0., 0., 1.]);

                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 0);
                    data.indicies.push(idx + 0);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 2);

                    idx = idx + 4;
                }

                if !neighbors[4].is_filled() {
                    // add face left
                    data.positions.push([fx, fy, fz]);
                    data.positions.push([fx, fy, fz + 1.]);
                    data.positions.push([fx, fy + 1., fz + 1.]);
                    data.positions.push([fx, fy + 1., fz]);

                    data.normals.push([-1., 0., 0.]);
                    data.normals.push([-1., 0., 0.]);
                    data.normals.push([-1., 0., 0.]);
                    data.normals.push([-1., 0., 0.]);

                    data.indicies.push(idx + 0);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 0);

                    idx = idx + 4;
                }

                if !neighbors[5].is_filled() {
                    // add face below
                    data.positions.push([fx, fy, fz]);
                    data.positions.push([fx + 1., fy, fz]);
                    data.positions.push([fx + 1., fy, fz + 1.]);
                    data.positions.push([fx, fy, fz + 1.]);

                    data.normals.push([0., -1., 0.]);
                    data.normals.push([0., -1., 0.]);
                    data.normals.push([0., -1., 0.]);
                    data.normals.push([0., -1., 0.]);

                    data.indicies.push(idx + 0);
                    data.indicies.push(idx + 1);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 2);
                    data.indicies.push(idx + 3);
                    data.indicies.push(idx + 0);

                    idx = idx + 4;
                }
            }
        }
    }

    // let mut i = 0;
    // for x in 0..(size_x + 1) {
    //     for z in 0..(size_z + 1) {
    //         let v1 = [x as f32, 0., z as f32];
    //         data.positions[i] = v1;
    //         data.normals[i] = [0., 1., 0.];
    //         i += 1;
    //     }
    // }

    // let mut ti = 0;
    // let mut vi = 0;

    // for _x in 0..size_z {
    //     for _z in 0..size_x {
    //         data.indicies[ti] = vi;
    //         data.indicies[ti + 2] = size_x + vi + 1; // first vertex of next row
    //         data.indicies[ti + 1] = vi + 1;

    //         data.indicies[ti + 3] = vi + 1;
    //         data.indicies[ti + 5] = size_x + vi + 1; // first vertex of next row
    //         data.indicies[ti + 4] = size_x + vi + 2; // second vertex of next row
    //         ti = ti + 6;
    //         vi = vi + 1;
    //     }
    //     vi = vi + 1;
    // }

    return Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, data.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals)
        .with_indices(Some(Indices::U32(data.indicies)));
}
