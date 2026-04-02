use bevy::prelude::*;

mod ahoy;
mod avian;
mod input;
mod mipmap;
mod seedling;
mod skein;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        avian::plugin,
        skein::plugin,
        ahoy::plugin,
        input::plugin,
        seedling::plugin,
        mipmap::plugin,
    ));
}
