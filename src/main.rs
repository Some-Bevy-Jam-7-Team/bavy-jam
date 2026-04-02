use bevy::asset::AssetMetaCheck;
use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::gltf::{GltfLoaderSettings, GltfPlugin};
use bevy::image::{ImageAddressMode, ImageSamplerDescriptor};
use bevy::prelude::*;

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
        .add_systems(Startup, setup_directional_light )
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
        .run()
}

fn setup(mut ns_app: ResMut<NextState<AppState>>) {
    ns_app.set(AppState::Loading);
}

fn load(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.insert_resource(AppAssets {
        landscape: asset_server.load_with_settings(
            "level.glb#Scene0",
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

pub fn default_image_sampler_descriptor() -> ImageSamplerDescriptor {
    ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        anisotropy_clamp: 16,
        ..ImageSamplerDescriptor::linear()
    }
}
