use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Resource)]
pub struct Brick;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image = asset_server.load("sprites/Button.png");
    for i in 0..5 {
        commands
            .spawn((
                Brick,
                Sprite {
                    image: image.clone(),
                    // Tint the sprite blue using Srgba (Standard RGB with Alpha)
                    color: Color::Srgba(Srgba::new(0.0, 0.0, 0.5 + i as f32 / 5.0, 1.0)),
                    ..default()
                },
                Transform::from_xyz(i as f32 * 200.0 - 400.0, 0.0, (i * 2) as f32),
                Pickable::default(), // Enables picking detection for the backend
            ))
            .with_children(|parent| {
                // 3. Spawn the Text2d as a child so it sits on top of the sprite
                parent.spawn((
                    Text2d::new("H"),
                    TextFont {
                        font: asset_server.load("fonts/JameGem08_2026-Regular.ttf").into(),
                        font_size: FontSize::Px(64.0), // Note: Bevy 0.19 requires FontSize::Px
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    // Offset the text relative to the sprite (slightly forward on the Z axis)
                    Transform::from_xyz(0.0, 0.0, (i * 2 + 1) as f32),
                    TextLayout::justify(Justify::Center),
                ));
            })
            // Use the modern `On<Event>` wrapper to completely bypass `Trigger`
            .observe(
                |drag: On<Pointer<Drag>>, mut query: Query<&mut Transform>| {
                    // The event target entity is safely accessed directly from the listener property
                    if let Ok(mut transform) = query.get_mut(drag.event_target()) {
                        transform.translation.x += drag.delta.x;
                        transform.translation.y -= drag.delta.y;
                    }
                },
            );
    }
}

pub fn rotate_system(_time: Res<Time>, mut query: Query<&mut Transform, With<Brick>>) {
    // Loop through all entities that have a Transform AND the SpinningPlayer component
    for (i, mut transform) in query.iter_mut().enumerate() {
        // Define the rotation speed in radians per second
        // let rotation_speed = 2.0;

        // let angle: f32 =
        // (((rotation_speed * time.elapsed().as_secs_f32()) % (2.0 * PI)) - PI).sin();
        //println!("time {} Angle: {}", time.elapsed().as_secs_f32(), angle);
        // Rotate around the local Z-axis (pointing out of the screen)
        // time.delta_seconds() ensures rotation is consistent regardless of framerate
        let angle = i as f32 * PI / 2.0;
        println!("I {} Angle: {}", i, angle);
        transform.rotation = Quat::from_rotation_x(angle);
    }
}
