use bevy::prelude::*;

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

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let image = asset_server.load("sprites/qr-code.png");
    // let front = asset_server.load("sprites/qr-code.png");
    let front = asset_server.load("sprites/cyan_white_top_left.png");
    let back = asset_server.load("sprites/blue.png");

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

    commands
        .spawn((Brick, FlipAngle::default(), Transform::default()))
        .with_children(|parent| {
            // Front face: sprite + text, shown while facing the camera
            parent.spawn((
                FrontFace,
                Sprite {
                    image: front.clone(),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.1), // Local offset from parent
            ));

            parent.spawn((
                FrontFace,
                Text2d::new("MT"),
                TextFont {
                    font: asset_server.load("fonts/JameGem08_2026-Regular.ttf").into(),
                    font_size: FontSize::Px(48.0), // Note: Bevy 0.19 requires FontSize::Px
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.2), // Offset the text relative to the sprite (slightly forward on the Z axis)
                TextLayout::justify(Justify::Center),
            ));

            // Back face: shown while facing away from the camera
            parent.spawn((
                BackFace,
                Sprite {
                    image: back.clone(),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.1), // Local offset from parent
            ));
        });
}

pub fn rotate_system(time: Res<Time>, mut query: Query<&mut FlipAngle, With<Brick>>) {
    for mut angle in &mut query {
        // Rotation speed: radians per second
        let x_speed = 0.5;
        let y_speed = 0.3;
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
