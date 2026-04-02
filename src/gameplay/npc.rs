use bevy::{color::palettes::tailwind, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, spawn).add_systems(Startup, config);
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub(crate) struct Npc;

fn config(mut gizmo_config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = gizmo_config_store.config_mut::<DefaultGizmoConfigGroup>();

    config.line.width = 4.;
}

fn spawn(mut gizmos: Gizmos, npcs: Query<(&Transform, &Name), With<Npc>>) {
    for (transform, name) in &npcs {
        let text = match name.as_str().split(".").next().unwrap_or(name.as_str()) {
            "Text 1" => "memes",
            _ => &format!(
                "Fuck, this text ({name}) is mislabeled. Uuuuuh pretend this is some really good content, plz"
            ),
        };
        gizmos.text(
            dbg!(Isometry3d::new(transform.translation, transform.rotation)),
            text,
            1.,
            Vec2::ZERO,
            tailwind::RED_800,
        );
    }
}
