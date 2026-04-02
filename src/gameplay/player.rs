use avian3d::prelude::*;
use bevy::{
    camera::Exposure,
    input::common_conditions::input_just_pressed,
    light::{Atmosphere, AtmosphereEnvironmentMapLight, atmosphere::ScatteringMedium},
    post_process::bloom::Bloom,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::{Press, *};

use crate::{LevelReady, PhysLayer};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SpeedBoost>();
    app.add_input_context::<PlayerInput>();
    app.add_observer(setup).add_systems(
        Update,
        (
            add_speed_boosts,
            capture_cursor.run_if(input_just_pressed(MouseButton::Left)),
            release_cursor.run_if(input_just_pressed(KeyCode::Escape)),
        ),
    );
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct SpeedBoost;

fn add_speed_boosts(
    mut cmd: Commands,
    q: Query<Entity, (With<RigidBody>, (Without<SpeedBoost>, Without<Player>))>,
) {
    q.iter().for_each(|e| {
        cmd.entity(e)
            .insert((
                SpeedBoost,
                CollisionEventsEnabled,
                Sensor,
                CollisionLayers::new(PhysLayer::Boost, PhysLayer::Player),
            ))
            .observe(boost_collision);
    });
}

fn boost_collision(
    trigger: On<CollisionEnd>,
    mut q_player: Query<&mut LinearVelocity, With<Player>>,
) {
    let other_entity = trigger.event().body2;
    let Some(mut boosted) = other_entity.and_then(|e| q_player.get_mut(e).ok()) else {
        return;
    };

    // have to test this a bit and find a good middle ground
    boosted.0 *= Vec3::splat(boost_value);

    // mb some audio?
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub(crate) struct Player;

fn setup(
    ready: On<LevelReady>,
    player: Query<(), With<Player>>,
    mut commands: Commands,
    mut media: ResMut<Assets<ScatteringMedium>>,
) {
    if !player.contains(ready.entity) {
        return;
    }
    // Spawn the player
    let player = commands
        .entity(ready.entity)
        .insert((
            // Add the character controller configuration. We'll use the default settings for now.
            CharacterController {
                speed: 16.0,
                friction_hz: 20.0,
                ..default()
            },
            // The KCC currently behaves best when using a cylinder
            Collider::cylinder(0.3, 1.8),
            // Configure inputs. The actions `Movement`, `Jump`, etc. are provided by Ahoy, you just need to bind them.
            PlayerInput,
            CollisionLayers::new(
                [PhysLayer::Default, PhysLayer::Player],
                [PhysLayer::Default, PhysLayer::Goal],
            ),
            actions!(PlayerInput[
                (
                    Action::<Movement>::new(),
                    DeadZone::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Axial::left_stick()
                    ))
                ),
                (
                    Action::<Jump>::new(),
                    Press::default(),
                    bindings![
                        KeyCode::Space,
                        GamepadButton::South,
                        Binding::mouse_wheel(),
                    ],
                ),
                (
                    Action::<Tac>::new(),
                    Press::default(),
                    bindings![
                        KeyCode::Space,
                        GamepadButton::South,
                        Binding::mouse_wheel(),
                    ],
                ),
                (
                    Action::<Crane>::new(),
                    Press::default(),
                    bindings![
                        KeyCode::Space,
                        GamepadButton::South,
                        Binding::mouse_wheel(),
                    ],
                ),
                (
                    Action::<Mantle>::new(),
                    Hold::new(0.2),
                    bindings![
                        KeyCode::Space,
                        GamepadButton::South,
                    ],
                ),
                (
                    Action::<Climbdown>::new(),
                    bindings![KeyCode::ControlLeft, GamepadButton::LeftTrigger2],
                ),
                (
                    Action::<Crouch>::new(),
                    bindings![KeyCode::ControlLeft, GamepadButton::LeftTrigger2],
                ),
                (
                    Action::<SwimUp>::new(),
                    bindings![KeyCode::Space, GamepadButton::South],
                ),
                (
                    Action::<RotateCamera>::new(),

                    Bindings::spawn((
                        Spawn((Binding::mouse_motion(), Scale::splat(0.07))),
                        Axial::right_stick().with((Scale::splat(4.0),  DeadZone::default())),
                    ))
                ),
            ]),
        ))
        .id();

    // Spawn the player camera
    commands.spawn((
        Camera3d::default(),
        Exposure::INDOOR,
        // Enable the optional builtin camera controller
        CharacterControllerCameraOf::new(player),
        Bloom::NATURAL,
        AtmosphereEnvironmentMapLight {
            intensity: 0.4,
            ..default()
        },
        Atmosphere::earth(media.add(ScatteringMedium::default())),
    ));
}

#[derive(Component, Default)]
pub(crate) struct PlayerInput;

fn capture_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn release_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}
