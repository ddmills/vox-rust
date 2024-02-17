use bevy::prelude::*;

pub struct TerrainPlugin;

const VOX_EMPTY: i8 = 0;
const VOX_STONE: i8 = 1;
const MAP_SIZE_X: usize = 16;
const MAP_SIZE_Y: usize = 16;
const MAP_SIZE_Z: usize = 32;

#[derive(Resource)]
struct Terrain {
    voxels: [[[i8; MAP_SIZE_Z]; MAP_SIZE_Y]; MAP_SIZE_X],
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        let t: Terrain = Terrain {
            voxels: [[[VOX_EMPTY; MAP_SIZE_Z]; MAP_SIZE_Y]; MAP_SIZE_X],
        };

        app.insert_resource(t)
            .add_systems(Startup, generate_voxels)
            .add_systems(Startup, debug_voxels);
    }
}

fn generate_voxels(mut terrain: ResMut<Terrain>) {
    for z in 24..MAP_SIZE_Z {
        for y in 0..MAP_SIZE_Y {
            for x in 0..MAP_SIZE_X {
                terrain.voxels[x][y][z] = VOX_STONE;
            }
        }
    }
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
