use bevy::{prelude::*, scene::SceneInstanceReady};

use crate::LevelReady;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn);
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub(crate) struct Npc;

fn spawn(ready: On<LevelReady>, npc: Query<&GlobalTransform, With<Npc>>) {
    let Ok(transform) = npc.get(ready.entity) else {
        return;
    };
    info!(?transform);
}
