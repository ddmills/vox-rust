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
        texture::{ImageLoaderSettings, ImageSampler},
    },
};

pub struct TerrainPlugin;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Block {
    Oob,
    Empty,
    Dirt,
    Stone,
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

    pub fn texture_id(&self) -> u32 {
        match *self {
            Block::Oob => 0,
            Block::Empty => 0,
            Block::Dirt => 1,
            Block::Stone => 2,
        }
    }
}

pub const MAP_SIZE_X: u16 = 32;
pub const MAP_SIZE_Z: u16 = 32;
pub const MAP_SIZE_Y: u16 = 32;

#[derive(Event)]
pub struct TerrainModifiedEvent;

#[derive(Resource)]
pub struct Terrain {
    pub slice: u16,
    pub blocks: [[[Block; MAP_SIZE_Y as usize]; MAP_SIZE_Z as usize]; MAP_SIZE_X as usize],
}

#[derive(Resource)]
pub struct TerrainMesh {
    mesh: Handle<Mesh>,
    material: Handle<TerrainMaterial>,
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            blocks: [[[Block::Empty; MAP_SIZE_Y as usize]; MAP_SIZE_Z as usize];
                MAP_SIZE_X as usize],
            slice: 18,
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
            .add_systems(Startup, (setup_terrain, setup_terrain_mesh).chain())
            .add_systems(Update, update_terrain);
    }
}

fn setup_terrain(
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

fn setup_terrain_mesh(
    mut commands: Commands,
    terrain: Res<Terrain>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let settings = |s: &mut ImageLoaderSettings| s.sampler = ImageSampler::nearest();
    let terrain_texture: Handle<Image> = asset_server.load_with_settings("terrain.png", settings);
    let slice = terrain.slice;
    let mesh_data = mesh_terrain_simple(&terrain);
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
    .with_inserted_attribute(ATTRIBUTE_PACKED_BLOCK, mesh_data.packed)
    .with_inserted_indices(Indices::U32(mesh_data.indicies));
    let handle = meshes.add(mesh);
    let material = materials.add(TerrainMaterial {
        color: Color::YELLOW_GREEN,
        texture: terrain_texture,
        texture_count: 4,
        terrain_slice_y: slice as u32,
    });

    commands.spawn((
        MaterialMeshBundle {
            mesh: handle.clone(),
            material: material.clone(),
            ..default()
        },
        Wireframe,
    ));

    let terrain_mesh = TerrainMesh {
        mesh: handle,
        material: material,
    };
    commands.insert_resource(terrain_mesh);
}

fn update_terrain(
    terrain: Res<Terrain>,
    terrain_mesh: Res<TerrainMesh>,
    mut ev_terrain_mod: EventReader<TerrainModifiedEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    if ev_terrain_mod.is_empty() {
        return;
    }
    ev_terrain_mod.clear();

    let mesh_data = mesh_terrain_simple(&terrain);
    let mesh = meshes.get_mut(&terrain_mesh.mesh).unwrap();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
    mesh.insert_attribute(ATTRIBUTE_PACKED_BLOCK, mesh_data.packed);
    mesh.insert_indices(Indices::U32(mesh_data.indicies));

    let mat = materials.get_mut(&terrain_mesh.material).unwrap();
    mat.terrain_slice_y = terrain.slice.clone() as u32;
}

const ATTRIBUTE_PACKED_BLOCK: MeshVertexAttribute =
    MeshVertexAttribute::new("PackedBlock", 9985136798, VertexFormat::Uint32);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform[2]]
    color: Color,
    #[uniform[3]]
    texture_count: u32,
    #[uniform[4]]
    terrain_slice_y: u32,
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
            ATTRIBUTE_PACKED_BLOCK.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

#[derive(Default)]
struct TerrainMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indicies: Vec<u32>,
    pub packed: Vec<u32>,
}

fn mesh_terrain_simple(terrain: &Res<Terrain>) -> TerrainMeshData {
    let mut data = TerrainMeshData::default();
    data.positions = vec![];
    data.normals = vec![];
    data.indicies = vec![];
    data.packed = vec![];

    let mut idx = 0;

    for x in 0..MAP_SIZE_X {
        for z in 0..MAP_SIZE_Z {
            for y in 0..terrain.slice {
                let block = terrain.get(x as i16, y as i16, z as i16);

                if !block.is_filled() {
                    continue;
                }

                let fx = x as f32;
                let fy = y as f32;
                let fz = z as f32;

                let neighbors = terrain.get_neighbors_immediate(x as i16, y as i16, z as i16);

                if y == (terrain.slice - 1) || !neighbors[0].is_filled() {
                    // add face above
                    data.positions.push([fx, fy + 1., fz]);
                    data.positions.push([fx + 1., fy + 1., fz]);
                    data.positions.push([fx + 1., fy + 1., fz + 1.]);
                    data.positions.push([fx, fy + 1., fz + 1.]);

                    data.packed.push(pack_block(block, FaceDir::PosY));
                    data.packed.push(pack_block(block, FaceDir::PosY));
                    data.packed.push(pack_block(block, FaceDir::PosY));
                    data.packed.push(pack_block(block, FaceDir::PosY));

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

                    data.packed.push(pack_block(block, FaceDir::NegZ));
                    data.packed.push(pack_block(block, FaceDir::NegZ));
                    data.packed.push(pack_block(block, FaceDir::NegZ));
                    data.packed.push(pack_block(block, FaceDir::NegZ));

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

                    data.packed.push(pack_block(block, FaceDir::PosX));
                    data.packed.push(pack_block(block, FaceDir::PosX));
                    data.packed.push(pack_block(block, FaceDir::PosX));
                    data.packed.push(pack_block(block, FaceDir::PosX));

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

                    data.packed.push(pack_block(block, FaceDir::PosZ));
                    data.packed.push(pack_block(block, FaceDir::PosZ));
                    data.packed.push(pack_block(block, FaceDir::PosZ));
                    data.packed.push(pack_block(block, FaceDir::PosZ));

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

                    data.packed.push(pack_block(block, FaceDir::NegX));
                    data.packed.push(pack_block(block, FaceDir::NegX));
                    data.packed.push(pack_block(block, FaceDir::NegX));
                    data.packed.push(pack_block(block, FaceDir::NegX));

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

                    data.packed.push(pack_block(block, FaceDir::NegY));
                    data.packed.push(pack_block(block, FaceDir::NegY));
                    data.packed.push(pack_block(block, FaceDir::NegY));
                    data.packed.push(pack_block(block, FaceDir::NegY));

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

    return data;
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FaceDir {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl FaceDir {
    pub fn bit(&self) -> u32 {
        match self {
            FaceDir::PosX => 0,
            FaceDir::NegX => 1,
            FaceDir::PosY => 2,
            FaceDir::NegY => 3,
            FaceDir::PosZ => 4,
            FaceDir::NegZ => 5,
        }
    }
}

fn pack_block(block: Block, dir: FaceDir) -> u32 {
    let t_id = block.texture_id(); // 0-15
    let f_id = dir.bit(); // 0-7

    return (t_id & 15) | ((f_id & 7) << 4);
}
