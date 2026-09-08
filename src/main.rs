use bevy::prelude::*;
use bevy_tween::prelude::*;

mod brick;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultTweenPlugins::default()))
        .add_tween_systems(
            PostUpdate,
            bevy_tween::component_tween_system::<brick::FlipAngleZ>(),
        )
        .add_systems(Startup, (setup, brick::setup))
        .add_systems(Update, brick::flip_faces_system)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the 2D Camera
    commands.spawn(Camera2d);
}
