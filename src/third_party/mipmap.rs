use bevy::prelude::*;
use bevy_mod_mipmap_generator::{MipmapGeneratorPlugin, generate_mipmaps};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MipmapGeneratorPlugin)
        .add_systems(Update, generate_mipmaps::<StandardMaterial>);
}
