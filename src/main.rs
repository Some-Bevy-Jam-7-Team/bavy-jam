use avian3d::prelude::{
    ColliderConstructor, CollisionEventsEnabled, CollisionLayers, CollisionStart, PhysicsLayer,
    Sensor,
};
use bevy::asset::AssetMetaCheck;
use bevy::color::palettes::css::BLACK;
use bevy::color::palettes::tailwind::GREEN_700;
use bevy::ecs::error::error;
use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::gltf::{GltfLoaderSettings, GltfPlugin};
use bevy::image::{ImageAddressMode, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy::window::PresentMode;

mod gameplay;
mod third_party;

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
        .set_error_handler(error)
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
                        rotate_scene_entity: false,
                        rotate_meshes: true,
                    },
                    ..default()
                }),
        )
        .add_plugins((third_party::plugin, gameplay::plugin))
        .init_state::<AppState>()
        .configure_sets(
            Update,
            (
                AppSet::Loading.run_if(in_state(AppState::Loading)),
                AppSet::InGame.run_if(in_state(AppState::InGame)),
            ),
        )
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
        .add_observer(setup_goals)
        .add_observer(fix_point_lights)
        .add_observer(fix_spot_lights)
        .add_observer(fix_directional_lights)
        .run()
}

fn setup(mut ns_app: ResMut<NextState<AppState>>) {
    ns_app.set(AppState::Loading);
}

fn load(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.insert_resource(AppAssets {
        landscape: asset_server.load_with_settings(
            "level.gltf#Scene0",
            |settings: &mut GltfLoaderSettings| {
                settings.load_lights = true;
                //settings.load_cameras = true;
            },
        ),
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

#[derive(PhysicsLayer, Default)]
pub enum PhysLayer {
    #[default]
    Default,
    Goal,
    Player,
}

/// Epic bevy impl uses box shadow on a mesh
fn setup_goals(
    ready: On<LevelReady>,
    mut commands: Commands,
    goals: Query<Entity, With<BoxShadow>>,
) {
    for goal_entity in goals {
        commands
            .entity(goal_entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((
                Sensor,
                ColliderConstructor::ConvexHullFromMesh,
                CollisionLayers::new(PhysLayer::Goal, PhysLayer::Player),
                CollisionEventsEnabled,
            ))
            .observe(on_goal_achieved);
    }
}

fn on_goal_achieved(_on: On<CollisionStart>, mut commands: Commands) {
    // win state :D
    commands.spawn((
        Node {
            width: vw(100.0),
            height: vh(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Node::default()
        },
        DespawnOnExit(AppState::InGame),
        BackgroundColor(BLACK.with_alpha(0.2).into()),
        children![(
            Text::new("YOU FRE*AKING WIN!"),
            TextFont {
                font_size: 72.0f32.into(),
                ..default()
            },
            TextColor(GREEN_700.into()),
        )],
    ));
}

fn spawn_landscape(mut cmd: Commands, assets: Res<AppAssets>) {
    cmd.spawn((Landscape, SceneRoot(assets.landscape.clone())))
        .observe(
            |ready: On<SceneInstanceReady>, children: Query<&Children>, mut commands: Commands| {
                for child in children.iter_descendants(ready.entity) {
                    commands.trigger(LevelReady { entity: child })
                }
            },
        );
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

fn fix_point_lights(
    ready: On<LevelReady>,
    mut commands: Commands,
    lights: Query<&PointLight>,
    transform_helper: TransformHelper,
) {
    let Ok(light) = lights.get(ready.entity) else {
        return;
    };
    commands.entity(ready.entity).despawn();
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            contact_shadows_enabled: true,
            ..*light
        },
        Transform::from(
            transform_helper
                .compute_global_transform(ready.entity)
                .unwrap(),
        ),
    ));
}

fn fix_spot_lights(add: On<Add, SpotLight>, mut lights: Query<&mut SpotLight>) {
    let Ok(mut light) = lights.get_mut(add.entity) else {
        return;
    };
    light.range = 100.0;
    light.shadow_maps_enabled = true;
}

fn fix_directional_lights(
    add: On<Add, DirectionalLight>,
    mut lights: Query<&mut DirectionalLight>,
) {
    let Ok(mut light) = lights.get_mut(add.entity) else {
        return;
    };
    light.illuminance = 6_000.;
    light.shadow_maps_enabled = true;
}

#[derive(Component, Default)]
pub struct Controller {
    pub enabled: bool,
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

#[derive(EntityEvent)]
struct LevelReady {
    entity: Entity,
}
