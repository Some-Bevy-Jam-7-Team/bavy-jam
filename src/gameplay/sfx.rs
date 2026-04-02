use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_ahoy::{CharacterControllerState, input::Jump};
use bevy_enhanced_input::prelude::Fire;
use bevy_seedling::{
    prelude::Volume,
    sample::{AudioSample, RandomPitch, SamplePlayer},
};
use rand::{Rng, SeedableRng};

use crate::gameplay::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, (walking, hello))
        .add_systems(
            Startup,
            (
                |server: Res<AssetServer>, mut commands: Commands| {
                    commands.insert_resource(Walkies(server.load("sfx/pop.ogg")));
                },
                ambience,
            ),
        )
        .add_observer(jumping);
}

#[derive(Resource)]
struct Walkies(Handle<AudioSample>);

struct WalkTimer(Timer);

impl Default for WalkTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

fn ambience(server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(
        SamplePlayer::new(server.load("sfx/rumble.ogg"))
            .looping()
            .with_volume(Volume::Decibels(-12.0)),
    );

    commands.spawn(
        SamplePlayer::new(server.load("sfx/crow_ambience.ogg"))
            .looping()
            .with_volume(Volume::Decibels(-18.0)),
    );
}

fn walking(
    player: Single<(&LinearVelocity, &CharacterControllerState), With<Player>>,
    time: Res<Time>,
    walkies: Res<Walkies>,
    mut timer: Local<WalkTimer>,
    mut commands: Commands,
) {
    let (velocity, controller) = player.into_inner();

    if velocity.0.length_squared() < 1.0 || controller.grounded.is_none() {
        timer.0.reset();
        return;
    }

    if timer.0.tick(time.delta()).just_finished() {
        commands.spawn((
            SamplePlayer::new(walkies.0.clone()).with_volume(Volume::Decibels(-6.0)),
            RandomPitch(2.0..3.5),
        ));
    }
}

fn jumping(
    _: On<Fire<Jump>>,
    player: Single<&CharacterControllerState, With<Player>>,
    server: Res<AssetServer>,
    mut commands: Commands,
) {
    if player.grounded.is_some() {
        commands.spawn((
            SamplePlayer::new(server.load("sfx/squeak.ogg")).with_volume(Volume::Decibels(3.0)),
            RandomPitch(1.0..1.5),
        ));
    }
}

struct HelloTimer(Timer);

impl Default for HelloTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

struct HelloRng(rand::rngs::SmallRng);

impl Default for HelloRng {
    fn default() -> Self {
        Self(rand::rngs::SmallRng::from_os_rng())
    }
}

fn hello(
    time: Res<Time>,
    server: Res<AssetServer>,
    mut timer: Local<HelloTimer>,
    mut rng: Local<HelloRng>,
    mut commands: Commands,
) {
    if timer.0.tick(time.delta()).just_finished() && rng.0.random_range(0..1000) == 0 {
        commands.spawn(
            SamplePlayer::new(server.load("sfx/hello.ogg")).with_volume(Volume::Decibels(-24.0)),
        );
    }
}
