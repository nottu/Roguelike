use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use specs::{
    prelude::*,
    saveload::{ConvertSaveload, SimpleMarkerAllocator},
    saveload::{DeserializeComponents, MarkedBuilder, Marker, SerializeComponents, SimpleMarker},
};
use specs_derive::{Component, ConvertSaveload};
use std::{
    convert::Infallible,
    fs::{self, File},
    path::Path,
};

use crate::{
    components::{
        AreaOfEffect, BlockedTile, CombatStats, Confusion, Consumable, FilePersistent, InBackpack,
        InflictsDamage, Item, Monster, Name, Position, ProvidesHealing, Ranged, Renderable,
        Viewshed,
    },
    map::Map,
    player::Player,
};

const SAVE_FILE: &str = "./save_game.json";

#[derive(Debug, Component, Clone, ConvertSaveload)]
pub struct SerializationHelper {
    map: Map,
}

macro_rules! serialize_individually {
    ($ecs: expr, $serializer: expr, $entities: expr, $markers: expr, $($type:ty),*) => {
        $(
        SerializeComponents::<Infallible, SimpleMarker<FilePersistent>>::serialize(
            &($ecs.read_storage::<$type>(),),
            $entities,
            $markers,
            $serializer,
        ).expect("Failed to serialize");
        )*
    };
}

pub fn save_game(ecs: &mut World) {
    let map_copy = ecs.get_mut::<Map>().expect("expected a map").clone();
    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map: map_copy })
        .marked::<SimpleMarker<FilePersistent>>()
        .build();

    {
        let save_file_path = Path::new(SAVE_FILE);
        let Ok(writer) = File::create(save_file_path) else {
            eprint!("Failed to create save_file={save_file_path:?}");
            return;
        };
        let mut serializer = ron::ser::Serializer::with_options(
            writer,
            Some(PrettyConfig::default()),
            ron::Options::default(),
        )
        .expect("failed to create ron serializer");
        // let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs,
            &mut serializer,
            &ecs.entities(),
            &ecs.read_storage::<SimpleMarker<FilePersistent>>(),
            Position,
            Renderable,
            Player, // should we save or just re-compute the viewshed?
            Viewshed,
            Monster,
            Name,
            BlockedTile,
            CombatStats,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            SerializationHelper
        );
    }

    ecs.delete_entity(save_helper)
        .expect("Failed to cleanup save_helper");
}

macro_rules! deserialize_individually {
    ($ecs: expr, $de: expr, $($type: ty),*) => {
        $(DeserializeComponents::<Infallible, SimpleMarker<FilePersistent>>::deserialize(
            &mut ( $ecs.write_storage::<$type>(), ),
            &mut $ecs.entities(), // entities
            &mut $ecs.write_storage::<SimpleMarker<FilePersistent>>(), // marker
            &mut $ecs.write_resource::<SimpleMarkerAllocator<FilePersistent>>(), // allocater
            $de,
        ).expect("Failed Deserializing");)
        *
    };
}

pub fn load_game(ecs: &mut World) {
    // Delete everything..
    ecs.delete_all();
    let data = fs::read_to_string(SAVE_FILE).expect("Expected Saved Game");
    let mut deserializer =
        ron::de::Deserializer::from_str(&data).expect("Failed to create Deserializer");

    deserialize_individually!(
        ecs,
        &mut deserializer,
        Position,
        Renderable,
        Player,
        // should we save or just re-compute the viewshed?
        Viewshed,
        Monster,
        Name,
        BlockedTile,
        CombatStats,
        Item,
        Consumable,
        Ranged,
        InflictsDamage,
        AreaOfEffect,
        Confusion,
        ProvidesHealing,
        InBackpack,
        SerializationHelper
    );

    let mut deleteme: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        // should only be one item
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<Map>();
            *worldmap = h.map.clone();
            worldmap.tile_content = vec![Vec::new(); super::map::MAP_SIZE];
            deleteme = Some(e);
        }
    }
    ecs.delete_entity(deleteme.unwrap())
        .expect("unable to delete helper");
}

pub fn delete_save() -> bool {
    if Path::new(SAVE_FILE).exists() {
        std::fs::remove_file(SAVE_FILE).expect("Failed to delete save file");
        true
    } else {
        false
    }
}
