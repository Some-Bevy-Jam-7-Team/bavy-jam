use bevy::prelude::*;
use bevy_ahoy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AhoyPlugins::default());
}
