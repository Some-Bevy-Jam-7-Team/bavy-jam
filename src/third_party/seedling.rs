use bevy::prelude::*;
use bevy_seedling::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(SeedlingPlugin {
        graph_config: bevy_seedling::prelude::GraphConfiguration::Empty,
        ..Default::default()
    })
    .add_systems(Startup, graph);
}

fn graph(mut commands: Commands) {
    let reverb_chain = commands
        .spawn(VolumeNode::from_decibels(-17.0))
        .chain_node(FreeverbNode {
            room_size: 0.96,
            damping: 0.5,
            width: 0.96,
            ..Default::default()
        })
        .head();

    commands
        .spawn((MainBus, VolumeNode::default()))
        .connect(AudioGraphOutput);

    // Pools
    commands
        .spawn((SamplerPool(DefaultPool), PoolSize(32..=64)))
        .connect(MainBus)
        .connect(reverb_chain);

    commands
        .spawn((SamplerPool(MusicPool), PoolSize(2..=2)))
        .connect(reverb_chain);
}
