use bevy::prelude::*;
use bevy_mod_mipmap_generator::{MipmapGeneratorPlugin, MipmapGeneratorSettings, generate_mipmaps};
use image::imageops::FilterType;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MipmapGeneratorPlugin)
        .insert_resource(MipmapGeneratorSettings {
            anisotropic_filtering: 16,
            filter_type: FilterType::Gaussian,
            ..Default::default()
        })
        .add_systems(Update, generate_mipmaps::<StandardMaterial>);
}
