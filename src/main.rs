use avian2d::debug_render::PhysicsDebugPlugin;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_tween::prelude::*;

mod brick;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(1920, 1080),
                    ..default()
                }),
                ..default()
            }),
            DefaultTweenPlugins::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .insert_resource(Gravity::ZERO)
        .add_tween_systems(
            PostUpdate,
            bevy_tween::component_tween_system::<brick::FlipAngleZ>(),
        )
        .add_systems(Startup, (setup, brick::setup))
        .add_systems(Update, (brick::flip_faces_system, toggle_physics_debug))
        .run();
}

fn setup(mut commands: Commands, mut gizmo_config_store: ResMut<GizmoConfigStore>) {
    // Spawn the 2D Camera
    commands.spawn(Camera2d);

    // Collider wireframes are off by default; toggle with F1.
    let (config, _) = gizmo_config_store.config_mut::<PhysicsGizmos>();
    config.enabled = false;
}

// Toggles the avian2d collider/wireframe debug rendering on F1.
fn toggle_physics_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut gizmo_config_store: ResMut<GizmoConfigStore>,
) {
    if keys.just_pressed(KeyCode::F1) {
        let (config, _) = gizmo_config_store.config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
    }
}
