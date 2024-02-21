use std::cmp::{max, min};

use bevy::{
    app::{Plugin, Startup, Update},
    ecs::{
        event::{EventReader, EventWriter},
        system::ResMut,
    },
    input::mouse::MouseWheel,
};

use crate::terrain::{Terrain, TerrainModifiedEvent, MAP_SIZE_Y};

pub struct SlicePlugin;

impl Plugin for SlicePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, scroll_events);
    }
}

fn scroll_events(
    mut scroll_evt: EventReader<MouseWheel>,
    mut terrain: ResMut<Terrain>,
    mut ev_terrain_mod: EventWriter<TerrainModifiedEvent>,
) {
    for ev in scroll_evt.read() {
        match ev.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                let scroll = ev.y as i16;
                let slice = terrain.slice as i16;
                let mut new_slice = slice + scroll;
                new_slice = max(0, new_slice);
                new_slice = min(new_slice, (MAP_SIZE_Y - 1) as i16);
                terrain.slice = new_slice as u16;

                println!(
                    "Scroll (line units): vertical: {}, horizontal: {}, slice: {}",
                    ev.y, ev.x, terrain.slice
                );
                ev_terrain_mod.send(TerrainModifiedEvent {});
            }
            bevy::input::mouse::MouseScrollUnit::Pixel => {
                println!(
                    "Scroll (pixel units): vertical: {}, horizontal: {}",
                    ev.y, ev.x
                );
            }
        }
    }
}
