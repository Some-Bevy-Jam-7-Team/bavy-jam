use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::asset::AssetMetaCheck;
use bevy::camera::Hdr;
use bevy::core_pipeline::prepass::DeferredPrepass;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::gltf::GltfPlugin;
use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::image::{ImageAddressMode, ImageSamplerDescriptor};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{Atmosphere, AtmosphereEnvironmentMapLight, ShadowFilteringMethod};
use bevy::pbr::{DefaultOpaqueRendererMethod, ScreenSpaceAmbientOcclusion};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

use bevy::render::view::ColorGrading;
use bevy::window::{CursorGrabMode, CursorOptions, PresentMode};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Setup,
    Loading,
    InGame,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum AppSet {
    Loading,
    InGame,
}

#[derive(Resource)]
struct AppAssets {
    landscape: Handle<Scene>,
}

fn main() -> AppExit {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bavy jam".to_string(),
                        fit_canvas_to_parent: true,
                        present_mode: PresentMode::Mailbox,
                        #[cfg(feature = "web")]
                        prevent_default_event_handling: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: default_image_sampler_descriptor(),
                })
                .set(GltfPlugin {
                    convert_coordinates: GltfConvertCoordinates {
                        rotate_scene_entity: true,
                        rotate_meshes: true,
                    },
                    ..default()
                }),
        )
        .init_state::<AppState>()
        .configure_sets(
            Update,
            (
                AppSet::Loading.run_if(in_state(AppState::Loading)),
                AppSet::InGame.run_if(in_state(AppState::InGame)),
            ),
        )
        .add_systems(Startup, (setup_camera, setup_directional_light))
        .add_systems(OnEnter(AppState::Setup), setup)
        .add_systems(OnEnter(AppState::Loading), (load, setup_loading_screen))
        .add_systems(OnEnter(AppState::InGame), spawn_landscape)
        .add_systems(
            Update,
            (
                set_in_game.run_if(assets_loaded).in_set(AppSet::Loading),
                main_loop.in_set(AppSet::InGame),
            ),
        )
        .add_systems(Update, (enable_cursor, disable_cursor, move_camera))
        .run()
}

fn setup(mut ns_app: ResMut<NextState<AppState>>) {
    ns_app.set(AppState::Loading);
}

fn load(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.insert_resource(AppAssets {
        landscape: asset_server.load("level.gltf#Scene0"),
    });
}

fn assets_loaded(assets: Res<AppAssets>, asset_server: Res<AssetServer>) -> bool {
    asset_server
        .get_load_state(assets.landscape.id())
        .is_some_and(|s| s.is_loaded())
}

fn set_in_game(mut ns_app: ResMut<NextState<AppState>>) {
    ns_app.set(AppState::InGame);
}

fn main_loop() {
    // TODO
}

#[derive(Component)]
#[require(Name::new("Landscape"))]
pub struct Landscape;

fn spawn_landscape(mut cmd: Commands, assets: Res<AppAssets>) {
    cmd.spawn((Landscape, SceneRoot(assets.landscape.clone())));
}

#[derive(Component)]
struct LoadingScreen;

fn setup_loading_screen(mut commands: Commands) {
    commands.spawn((
        Node {
            height: percent(100),
            width: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        DespawnOnExit(AppState::Loading),
        BackgroundColor(Color::BLACK),
        LoadingScreen,
        children![(Text::new("Loading..."), TextFont::from_font_size(67.0))],
    ));
}

fn disable_cursor(
    btn: Res<ButtonInput<MouseButton>>,
    mut q: Query<&mut CursorOptions, With<Window>>,
    controller: Single<&mut Controller>,
) {
    if !btn.just_pressed(MouseButton::Left) {
        return;
    };

    for mut options in &mut q {
        options.grab_mode = CursorGrabMode::Locked;
        options.visible = false;
    }

    let mut controller = controller.into_inner();
    controller.enabled = true;
}

fn enable_cursor(
    key: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut CursorOptions, With<Window>>,
    controller: Single<&mut Controller>,
) {
    if !key.just_pressed(KeyCode::Escape) {
        return;
    };

    for mut options in &mut q {
        options.grab_mode = CursorGrabMode::None;
        options.visible = true;
    }

    let mut controller = controller.into_inner();
    controller.enabled = false;
}

pub fn setup_camera(
    mut cmd: Commands,
    q_camera: Query<&Camera>,

    mut media: ResMut<Assets<ScatteringMedium>>,
) {
    if !q_camera.is_empty() {
        return;
    };

    cmd.spawn((
        (Camera::default(), Camera3d::default()),
        Controller::default(),
        Hdr,
        ColorGrading::default(),
        Bloom::NATURAL,
        Tonemapping::TonyMcMapface,
        Transform::from_xyz(-30., 20., 30.).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        ShadowFilteringMethod::Temporal,
        DeferredPrepass,
        AtmosphereEnvironmentMapLight {
            intensity: 0.4,
            ..default()
        },
        Atmosphere::earth(media.add(ScatteringMedium::default())),
    ));
}

fn setup_directional_light(mut cmd: Commands) {
    cmd.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            contact_shadows_enabled: true,
            illuminance: 10_000.,
            color: Color::srgb(1.0, 0.98, 0.95),
            ..default()
        },
        Transform::from_xyz(2., 2., 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Component, Default)]
pub struct Controller {
    pub enabled: bool,
}

fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    single: Single<(&mut Transform, &Controller)>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let (mut transform, controller) = single.into_inner();

    if !controller.enabled {
        return;
    };

    let amplify = keyboard_input.pressed(KeyCode::ShiftLeft);

    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += transform.forward().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction += transform.back().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction += transform.left().as_vec3();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += transform.right().as_vec3();
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();

        let factor = if amplify { 40.0 } else { 10.0 };

        transform.translation += direction * factor * time.delta_secs();
    }

    let delta = accumulated_mouse_motion.delta;

    if delta != Vec2::ZERO {
        let delta_yaw = -delta.x * 0.001;
        let delta_pitch = -delta.y * 0.001;

        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;

        const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
        let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

pub fn default_image_sampler_descriptor() -> ImageSamplerDescriptor {
    ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        anisotropy_clamp: 16,
        ..ImageSamplerDescriptor::linear()
    }
}
