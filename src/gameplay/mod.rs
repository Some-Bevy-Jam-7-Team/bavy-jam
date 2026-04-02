use bevy::prelude::*;
mod npc;
mod player;
mod sfx;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((player::plugin, npc::plugin, sfx::plugin));
}
