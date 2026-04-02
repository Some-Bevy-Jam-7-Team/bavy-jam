use bevy::prelude::*;
use bevy_skein::SkeinPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(SkeinPlugin { handle_brp: true });
}
