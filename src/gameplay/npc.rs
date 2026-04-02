use bevy::{color::palettes::tailwind, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, spawn).add_systems(Startup, config);
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub(crate) struct Npc;

fn config(mut gizmo_config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = gizmo_config_store.config_mut::<DefaultGizmoConfigGroup>();

    config.line.width = 7.;
}

fn spawn(mut gizmos: Gizmos, npcs: Query<(&Transform, &TextSpan), With<Npc>>) {
    for (transform, text) in &npcs {
        gizmos.text(
            Isometry3d::new(transform.translation, transform.rotation),
            text.as_str(),
            1.,
            Vec2::ZERO,
            tailwind::RED_800,
        );
    }
}
