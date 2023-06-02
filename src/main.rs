mod extractor;
mod processor;
mod map_colours;
mod resource_pack;
mod http_server;
pub mod util;

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use ffmpeg::format;
use log::info;
use processor::FrameProcessor;
use tokio::runtime::Runtime;
use valence::client::chat::ChatMessageEvent;
use valence::entity::{ObjectData, item_frame};
use valence::entity::player::PlayerEntityBundle;
use valence::glam::ivec3;
use valence::protocol::Encode;
use valence::protocol::encode::WritePacket;
use valence::protocol::packet::map::{MapUpdateS2c, self};
use valence::protocol::packet::sound::{PlaySoundS2c, SoundId, SoundCategory};
use valence::protocol::var_int::VarInt;
use valence::prelude::*;
use valence::entity::glow_item_frame::GlowItemFrameEntityBundle;

extern crate ffmpeg_next as ffmpeg;

fn main() {
    env_logger::init();
    info!("Staring...");

    let args: Vec<String> = env::args().collect();

    let input = format::input(&args.get(1).expect("Cannot open video file")).expect("Unable to load video");
    let width = args[2].parse::<u32>().unwrap();
    let height = args[3].parse::<u32>().unwrap();
    let local_url = args.get(4).unwrap();
    let clip_length = args[5].parse::<usize>().unwrap();

    ffmpeg::init().expect("Could not initialise Ffmpeg runtime");
    info!("Initialised ffmpeg");

    let pack = resource_pack::create(&extractor::extract_audio(args.get(1).unwrap(), clip_length)).unwrap();
    info!("Created resource pack");

    // Spawn http server
    let rt = Runtime::new().unwrap();
    rt.spawn(async move {
        http_server::serve(pack.clone()).await;  
    });


    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(init_clients)
        .add_system(on_chat_message)
        .add_system(despawn_disconnected_clients)
        .insert_non_send_resource(FrameProcessor::new(input, width, height, clip_length))
        .add_system(update_screen)
        .insert_resource(LocalURL(local_url.to_owned()))
        .run();
}

#[derive(Resource)]
struct LocalURL(String);

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Query<&DimensionType>,
    biomes: Query<&Biome>,
    processor: NonSend<FrameProcessor>,
) {
    let mut instance = Instance::new(Ident::new("overworld").unwrap().to_string_ident(), &dimensions, &biomes, &server);

    let width = processor.width as i32;
    let height = processor.height as i32;

    let width_maps = num_integer::div_ceil(width, 128);
    let height_maps = num_integer::div_ceil(height, 128);

    let width_chunks = num_integer::div_ceil(width_maps, 16);
    let height_chunks = num_integer::div_ceil(height_maps, 16);

    // Initialise chunks
    for x in -1..=width_chunks {
        for z in -1..=height_chunks {
            instance.insert_chunk([x, z], Chunk::default());
        }
    }

    // Set screen blocks
    for x in 0..width_maps {
        for z in 0..height_maps {
            instance.set_block([x, 63, z], BlockState::BLACK_CONCRETE);
        }
    }

    let instance_id = commands.spawn(instance).id();

    // Spawn item frames
    for x in 0..width_maps {
        for z in 0..height_maps {
            let mut nbt = Compound::new();
            nbt.insert("map", z*width_maps + x);

            commands.spawn(GlowItemFrameEntityBundle {
                location: Location(instance_id),
                position: Position(DVec3::new(x as f64, 64., z as f64)),
                item_frame_item_stack: item_frame::ItemStack(ItemStack::new(ItemKind::FilledMap, 1, Some(nbt))),
                object_data: ObjectData(1),
                ..Default::default()
            });
        }
    }

    info!("Initialised Minecraft server");
}

fn init_clients(
    mut clients: Query<(Entity, &UniqueId, &mut Client, &mut GameMode), Added<Client>>,
    instances: Query<Entity, With<Instance>>,
    mut commands: Commands,
    local_url: Res<LocalURL>
) {
    for (entity, uuid, mut client, mut game_mode) in &mut clients {
        *game_mode = GameMode::Creative;

        client.send_message("Welcome to MCVideo V3!");
        client.set_resource_pack(&format!(
            "http://{}:25566#{}",
            
            // Hacky way of forcing the client to clear the resource pack cache
            // https://bugs.mojang.com/browse/MC-164316
            local_url.0, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 100000
        ), "", true, None);

        let mut brand = Vec::new();
        "MCVideo".encode(&mut brand).unwrap();
        client.send_custom_payload( Ident::new("minecraft:brand").unwrap().as_str_ident(), &brand);

        commands.entity(entity).insert(PlayerEntityBundle {
            location: Location(instances.single()),
            position: Position(DVec3::new(0., 64., 0.)),
            uuid: *uuid,
            ..Default::default()
        });
    }
}

fn update_screen(
    mut processor: NonSendMut<FrameProcessor>,
    mut clients: Query<&mut Client>,
) {
    if clients.is_empty() || processor.paused { return; }

    if let Some(frame) = processor.next() {
        if let Some(f) = frame {
            for mut client in clients.iter_mut() {
                for (i, map_data) in f.iter().enumerate() {
                    client.write_packet(&MapUpdateS2c {
                        scale: 0,
                        icons: None,
                        locked: false,
                        map_id: VarInt(i as i32),
                        data: Some(map::Data {
                            columns: 128,
                            rows: 128,
                            position: [0, 0],
                            data: map_data
                        })
                    });
                }
            }
        }

        let (should_play, index) = processor.should_play_audio();
        if should_play {
            for mut client in clients.iter_mut() {
                play_clip_sound(&mut client, index);
            }
        }
    } else {
        // info!("Video finished playing");
        // process::exit(0);
    }

    // info!("update: {}ms\tct: {}\ttps: {}\tfps: {}", start.elapsed().as_millis(), server.current_tick(), server.tps(), (server.current_tick() - start_time.tick) as f32 / start_time.time.elapsed().as_secs_f32());
}

fn on_chat_message(
    mut events: EventReader<ChatMessageEvent>,
    mut clients: Query<&mut Client>,
    mut processor: NonSendMut<FrameProcessor>,
) {
    for event in events.iter() {
        let Ok(mut client) = clients.get_mut(event.client) else { continue; };

        if &*event.message == "!play" {
            if processor.should_play_audio().0 {
                play_clip_sound(&mut client, 0);
            }

            processor.start();

            client.send_message("Started playing video");
        }
    }
}

fn play_clip_sound(client: &mut Client, clip_index: usize) {
    client.write_packet(&PlaySoundS2c {
        id: SoundId::Direct {
            id: Ident::new(format!("audio:{}", clip_index)).unwrap(),
            range: Some(0.)
        },
        category: SoundCategory::Master,
        position: ivec3(0, 64, 0),
        volume: 100.,
        pitch: 1.,
        seed: 0,
    });

    info!("Played audio clip {clip_index}");
}