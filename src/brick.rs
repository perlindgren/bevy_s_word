use bevy::prelude::*;
use bevy_tween::{
    combinator::{parallel, tween},
    prelude::*,
};
use std::f32::consts::{PI, TAU};

#[derive(Component)]
pub struct Brick;

#[derive(Component)]
pub struct FrontFace;

#[derive(Component)]
pub struct BackFace;

// Logical flip angles for a Brick, in radians. Driving the visual squish via
// Transform::scale (instead of a real 3D rotation) keeps each child's Z
// constant, since Text2d recomputes its glyph Z from the fully-rotated
// transform and would otherwise desync from the front sprite's Z at large
// angles (glyph offsets are much larger than our small Z gap).
#[derive(Component, Default)]
pub struct FlipAngle {
    /// Rotation around the local X-axis (squishes the Y scale, top-to-bottom flip).
    pub x: f32,
    /// Rotation around the local Y-axis (squishes the X scale, left-to-right flip).
    pub y: f32,
    /// Rotation around the local Z-axis (in-plane spin, applied as a real rotation).
    pub z: f32,
}

#[derive(Component, Default)]
struct DragTracker {
    pub was_dragged: bool,
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let front = asset_server.load("sprites/Button.png");
    let back = asset_server.load("sprites/Button.png");
    let font = asset_server.load("fonts/JameGem08_2026-Regular.ttf");

    for i in 0..4 {
        brick(
            &mut commands,
            front.clone(),
            back.clone(),
            font.clone().into(),
            Color::WHITE,
            Color::BLACK,
            'A',
            Transform::from_xyz((i - 2) as f32 * 200.0, 0.0, i as f32 * 0.1),
            FlipAngle {
                x: 0.0,
                y: PI,
                z: i as f32 * TAU / 4.0,
            },
        );
    }
}

fn brick(
    commands: &mut Commands,
    front: Handle<Image>,
    back: Handle<Image>,
    font: FontSource,
    brick_color: Color,
    text_color: Color,
    text: char,
    transform: Transform,
    angle: FlipAngle,
) {
    commands
        .spawn((
            Brick,
            angle,
            DragTracker::default(),
            Visibility::default(),
            transform,
        ))
        .with_children(|parent| {
            // Front face: sprite + text, shown while facing the camera
            parent.spawn((
                FrontFace,
                Sprite {
                    image: front.clone(),
                    color: brick_color,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.01), // Local offset from parent
                Pickable::default(),
            ));

            parent.spawn((
                FrontFace,
                Text2d::new(text.to_string()),
                TextFont {
                    font,
                    font_size: FontSize::Px(64.0), // Note: Bevy 0.19 requires FontSize::Px
                    ..default()
                },
                TextColor(text_color),
                Transform::from_xyz(0.0, 0.0, 0.02), // Offset the text relative to the sprite (slightly forward on the Z axis)
                TextLayout::justify(Justify::Center),
                DragTracker::default(),
            ));

            // Back face: shown while facing away from the camera
            parent.spawn((
                BackFace,
                Sprite {
                    image: back.clone(),
                    color: brick_color.darker(0.2),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.01), // Local offset from parent
                Pickable::default(),
            ));
        })
        .observe(
            |mut drag: On<Pointer<Drag>>,
             mut query_transform: Query<&mut Transform>,
             mut query_drag: Query<&mut DragTracker>| {
                // The event target entity is safely accessed directly from the listener property
                println!(
                    "Entity {:?} was dragged by pointer {:?}",
                    drag.event_target(),
                    drag.pointer_id
                );
                if let Ok(mut drag_tracker) = query_drag.get_mut(drag.event_target()) {
                    drag_tracker.was_dragged = true;
                }
                if let Ok(mut transform) = query_transform.get_mut(drag.event_target()) {
                    transform.translation.x += drag.delta.x;
                    transform.translation.y -= drag.delta.y;
                };
                drag.propagate(false);
            },
        )
        // .observe(
        //     |mut click: On<Pointer<Click>>, mut query: Query<&mut DragTracker>| {
        //         // Read event details
        //         println!(
        //             "Entity {:?} was clicked by pointer {:?}",
        //             click.event_target(),
        //             click.pointer_id
        //         );
        //         if let Some(mut drag_tracker) = query.single_mut().ok() {
        //             drag_tracker.was_dragged = true;
        //         }
        //         // If you want to prevent this click from bubbling up to parent entities:
        //         click.propagate(true);
        //     },
        // )
        .observe(
            |mut click: On<Pointer<Release>>, mut query: Query<&mut DragTracker>| {
                // Read event details
                println!(
                    "Entity {:?} was released by pointer {:?}",
                    click.event_target(),
                    click.pointer_id
                );

                if let Ok(mut drag_tracker) = query.get_mut(click.event_target()) {
                    if !drag_tracker.was_dragged {
                        println!("Click.");
                        // let tween = Tween::new(
                        //     EaseFunction::QuadraticOut,
                        //     Duration::from_millis(150), // Fast, snappy flip
                        //     TransformScaleLens {
                        //         start: transform.scale,
                        //         end: Vec3::new(-1.0, 1.0, 1.0),
                        //     },
                        // );

                        // // Insert Animator to override or replace the current scale
                        // commands.entity(player_entity).insert(Animator::new(tween));
                    }
                    drag_tracker.was_dragged = false;
                } else {
                    println!("no track")
                }

                // If you want to prevent this click from bubbling up to parent entities:
                click.propagate(true);
            },
        );
}

pub fn rotate_system(time: Res<Time>, mut query: Query<&mut FlipAngle, With<Brick>>) {
    for mut angle in &mut query {
        // Rotation speed: radians per second
        let x_speed = 0.0;
        let y_speed = 0.0;
        let z_speed = 0.2;

        angle.x += x_speed * time.delta_secs();
        angle.y += y_speed * time.delta_secs();
        angle.z += z_speed * time.delta_secs();
    }
}

// An orthographic camera can't distinguish a true 3D rotation of a flat
// quad from simply scaling its X/Y axes by cos(angle) - both produce the same
// squish. Using scale keeps every child's Z fixed, so Text2d's glyph Z
// (which Bevy derives by rotating each glyph's local offset through the full
// transform) never drifts away from the front sprite's Z.
pub fn flip_faces_system(
    mut bricks: Query<(&FlipAngle, &mut Transform, &Children), With<Brick>>,
    mut faces: Query<
        (&mut Transform, &mut Visibility, Has<FrontFace>),
        (Or<(With<FrontFace>, With<BackFace>)>, Without<Brick>),
    >,
) {
    for (angle, mut brick_transform, children) in &mut bricks {
        let squish_x = angle.y.cos();
        let squish_y = angle.x.cos();
        // An odd number of axis flips reveals the back face.
        let facing_camera = squish_x * squish_y > 0.0;
        // Z is a genuine in-plane rotation (roll), so it's applied to the parent directly.
        brick_transform.rotation = Quat::from_rotation_z(angle.z);
        for &child in children {
            if let Ok((mut transform, mut visibility, is_front)) = faces.get_mut(child) {
                transform.scale.x = squish_x;
                transform.scale.y = squish_y;
                *visibility = if is_front == facing_camera {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

// for i in 0..1 {
//     commands
//         .spawn((
//             Brick,
//             Sprite {
//                 image: image.clone(),
//                 // Tint the sprite blue using Srgba (Standard RGB with Alpha)
//                 color: Color::Srgba(Srgba::new(0.0, 0.0, 0.5 + i as f32 / 5.0, 1.0)),
//                 ..default()
//             },
//             Transform::from_xyz(i as f32 * 200.0 - 400.0, 0.0, 0 as f32),
//             Pickable::default(), // Enables picking detection for the backend
//         ))
//         .with_children(|parent| {
//             // 3. Spawn the Text2d as a child so it sits on top of the sprite
//             parent.spawn((
//                 Text2d::new("H"),
//                 TextFont {
//                     font: asset_server.load("fonts/JameGem08_2026-Regular.ttf").into(),
//                     font_size: FontSize::Px(0.0), // Note: Bevy 0.19 requires FontSize::Px
//                     ..default()
//                 },
//                 TextColor(Color::WHITE),
//                 // Offset the text relative to the sprite (slightly forward on the Z axis)
//                 Transform::from_xyz(0.0, 0.0, 0.1),
//                 TextLayout::justify(Justify::Center),
//             ));
//         })
//         // Use the modern `On<Event>` wrapper to completely bypass `Trigger`
//         .observe(
//             |drag: On<Pointer<Drag>>, mut query: Query<&mut Transform>| {
//                 // The event target entity is safely accessed directly from the listener property
//                 if let Ok(mut transform) = query.get_mut(drag.event_target()) {
//                     transform.translation.x += drag.delta.x;
//                     transform.translation.y -= drag.delta.y;
//                 }
//             },
//         );
// }
