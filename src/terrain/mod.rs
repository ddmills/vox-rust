use bevy::{
    pbr::{wireframe::Wireframe, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, MeshVertexBufferLayout},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PrimitiveTopology, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, VertexFormat,
        },
        Render,
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

    pub fn texture_id(&self) -> u32 {
        match *self {
            Block::Oob => 0,
            Block::Empty => 0,
            Block::Dirt => 1,
            Block::Stone => 0,
        }
    }
}

const MAP_SIZE_X: u16 = 32;
const MAP_SIZE_Z: u16 = 32;
const MAP_SIZE_Y: u16 = 32;

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
            for y in 0..MAP_SIZE_Y {
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
    // for y in 0..MAP_SIZE_Y {
    //     println!("y={}", y);
    //     for z in 0..MAP_SIZE_Z {
    //         for x in 0..MAP_SIZE_X {
    //             let d = terrain.get(x, y, z);
    //             print!("{}", d);
    //         }
    //         println!("");
    //     }
    // }
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
    mut materials: ResMut<Assets<TerrainMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if ev_terrain_mod.is_empty() {
        return;
    }

    ev_terrain_mod.clear();
    let terrain_texture: Handle<Image> = asset_server.load("terrain.png");

    let mesh = meshes.add(mesh_terrain_simple(&terrain));
    // let dirt = materials.add(Color::rgb_u8(255, 144, 100).into());
    let mat = materials.add(TerrainMaterial {
        color: Color::YELLOW_GREEN,
    });

    commands.spawn((
        MaterialMeshBundle {
            mesh: mesh.clone(),
            material: mat,
            ..default()
        },
        // Wireframe,
    ));
}

const ATTRIBUTE_TEXTURE_IDX: MeshVertexAttribute =
    MeshVertexAttribute::new("TextureIdx", 988540917, VertexFormat::Uint32);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {
    #[uniform[0]]
    color: Color,
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_TEXTURE_IDX.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

#[derive(Default)]
struct TerrainMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_idx: Vec<u32>,
    pub indicies: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
}

fn mesh_terrain_simple(terrain: &Res<Terrain>) -> Mesh {
    let mut data = TerrainMeshData::default();
    data.positions = vec![];
    data.normals = vec![];
    data.tex_idx = vec![];
    data.indicies = vec![];
    data.uvs = vec![];

    let mut idx = 0;

    for x in 0..MAP_SIZE_X {
        for z in 0..MAP_SIZE_Z {
            for y in 0..MAP_SIZE_Y {
                let block = terrain.get(x as i16, y as i16, z as i16);

                if !block.is_filled() {
                    continue;
                }

                let fx = x as f32;
                let fy = y as f32;
                let fz = z as f32;

                let neighbors = terrain.get_neighbors_immediate(x as i16, y as i16, z as i16);

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

                    data.uvs.push([0., 0.]);
                    data.uvs.push([0.5, 0.]);
                    data.uvs.push([0.5, 0.5]);
                    data.uvs.push([0., 0.5]);

                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
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

                    data.uvs.push([0., 0.]);
                    data.uvs.push([0.5, 0.]);
                    data.uvs.push([0.5, 0.5]);
                    data.uvs.push([0., 0.5]);

                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
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

                    data.uvs.push([0., 0.]);
                    data.uvs.push([0.5, 0.]);
                    data.uvs.push([0.5, 0.5]);
                    data.uvs.push([0., 0.5]);

                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
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

                    data.uvs.push([0., 0.]);
                    data.uvs.push([0.5, 0.]);
                    data.uvs.push([0.5, 0.5]);
                    data.uvs.push([0., 0.5]);

                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
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

                    data.uvs.push([0., 0.]);
                    data.uvs.push([0.5, 0.]);
                    data.uvs.push([0.5, 0.5]);
                    data.uvs.push([0., 0.5]);

                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
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

                    data.uvs.push([0., 0.]);
                    data.uvs.push([0.5, 0.]);
                    data.uvs.push([0.5, 0.5]);
                    data.uvs.push([0., 0.5]);

                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    data.tex_idx.push(block.texture_id());
                    idx = idx + 4;
                }
            }
        }
    }

    return Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, data.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, data.uvs)
    .with_inserted_attribute(ATTRIBUTE_TEXTURE_IDX, data.tex_idx)
    .with_inserted_indices(Indices::U32(data.indicies));
}
