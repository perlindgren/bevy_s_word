use bevy::prelude::*;
use bevy_tween::prelude::*;

mod brick;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultTweenPlugins::default()))
        .add_systems(Startup, (setup, brick::setup))
        .add_systems(
            Update,
            (brick::rotate_system, brick::flip_faces_system).chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn the 2D Camera
    commands.spawn(Camera2d);
}
