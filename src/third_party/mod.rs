use bevy::prelude::*;

mod ahoy;
mod avian;
mod input;
mod skein;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((avian::plugin, skein::plugin, ahoy::plugin, input::plugin));
}
