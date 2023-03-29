mod extractor;
mod processor;
mod map_colours;

use std::{env, time::Instant, process};

use ffmpeg::format;
use processor::FrameProcessor;
use valence::{prelude::*, entity::{player::PlayerBundle, glow_item_frame::GlowItemFrameBundle, item_frame, ObjectData}, protocol::{Encode, packet::s2c::play::{MapUpdateS2c, map_update::Data}, var_int::VarInt}, packet::WritePacket};

extern crate ffmpeg_next as ffmpeg;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let args: Vec<String> = env::args().collect();

    ffmpeg::init().expect("Could not initialise Ffmpeg runtime");

    let input = format::input(&args.get(1).expect("Cannot open video file")).expect("Unable to load video");
    let width = args[2].parse::<u32>().unwrap();
    let height = args[3].parse::<u32>().unwrap();

    App::new()
        .add_plugin(ServerPlugin::new(())
            .with_max_connections(10)
            .with_tick_rate(30))
        .add_startup_system(setup)
        .add_system(init_clients)
        .add_system(default_event_handler.in_schedule(EventLoopSchedule))
        .add_systems(PlayerList::default_systems())
        .add_system(despawn_disconnected_clients)
        .insert_non_send_resource(FrameProcessor::new(input, width, height))
        .add_system(update_screen)
        .insert_resource(StartTime { time: Instant::now(), tick: 0 })
        .run();
}

#[derive(Resource)]
struct StartTime {
    time: Instant,
    tick: i64,
}

fn setup(mut commands: Commands, server: Res<Server>, processor: NonSend<FrameProcessor>) {
    let mut instance = server.new_instance(DimensionId::default());

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

            commands.spawn(GlowItemFrameBundle {
                location: Location(instance_id),
                position: Position(DVec3::new(x as f64, 64., z as f64)),
                item_frame_item_stack: item_frame::ItemStack(ItemStack::new(ItemKind::FilledMap, 1, Some(nbt))),
                object_data: ObjectData(1),
                ..Default::default()
            });
        }
    }
}

fn init_clients(
    mut clients: Query<(Entity, &UniqueId, &mut Client, &mut GameMode), Added<Client>>,
    instances: Query<Entity, With<Instance>>,
    mut commands: Commands,
    mut start_time: ResMut<StartTime>,
    server: Res<Server>,
    mut processor: NonSendMut<FrameProcessor>,
) {
    for (entity, uuid, mut client, mut game_mode) in &mut clients {
        processor.start();
        *start_time = StartTime { time: Instant::now(), tick: server.current_tick() } ;

        *game_mode = GameMode::Creative;

        client.send_message("Welcome to MCVideo V3!");

        let mut brand = Vec::new();
        "MCVideo".encode(&mut brand).unwrap();
        client.send_custom_payload( Ident::new("minecraft:brand").unwrap().as_str_ident(), &brand);

        commands.entity(entity).insert(PlayerBundle {
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
    server: Res<Server>,
    start_time: Res<StartTime>
) {
    if clients.is_empty() { return; }

    let start = Instant::now();
    if let Some(frame) = processor.next() {
        println!("Processing took: {}ms", start.elapsed().as_millis());
        for mut client in clients.iter_mut() {
            for (i, map_data) in frame.iter().enumerate() {
                client.write_packet(&MapUpdateS2c {
                    scale: 0,
                    icons: None,
                    locked: false,
                    map_id: VarInt(i as i32),
                    data: Some(Data {
                        columns: 128,
                        rows: 128,
                        position: [0, 0],
                        data: map_data
                    })
                });
            }
        }
    } else {
        println!("Video finished playing");
        process::exit(0);
    }

    // println!("update: {}ms\tct: {}\ttps: {}\tfps: {}", start.elapsed().as_millis(), server.current_tick(), server.tps(), (server.current_tick() - start_time.tick) as f32 / start_time.time.elapsed().as_secs_f32());
}