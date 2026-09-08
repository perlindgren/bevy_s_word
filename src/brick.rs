use bevy::prelude::*;
use bevy_tween::{combinator::tween, prelude::*};
use rand::Rng;
use std::f32::consts::{PI, TAU};
use std::time::Duration;

/// Radians per second the in-plane spin (`FlipAngle::z`) rotates at, driven by the tween below.
const Z_SPIN_SPEED: f32 = 0.2;

/// [`Interpolator`] for [`FlipAngle::z`], mirroring `bevy_tween`'s built-in `AngleZ`.
#[derive(Debug, Clone, Copy)]
pub struct FlipAngleZ {
    pub start: f32,
    pub end: f32,
}

impl Interpolator for FlipAngleZ {
    type Item = FlipAngle;

    fn interpolate(&self, item: &mut Self::Item, value: f32, _previous_value: f32) {
        item.z = self.start.lerp(self.end, value);
    }
}

/// Constructor for [`FlipAngleZ`]
pub fn flip_angle_z(start: f32, end: f32) -> FlipAngleZ {
    FlipAngleZ { start, end }
}

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

/// Number of bricks spawned at random positions within the window.
const BRICK_COUNT: usize = 4;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    let front = asset_server.load("sprites/Button.png");
    let back = asset_server.load("sprites/Button.png");
    let font = asset_server.load("fonts/JameGem08_2026-Regular.ttf");

    let (half_width, half_height) = windows
        .single()
        .map(|window| (window.width() / 2.0, window.height() / 2.0))
        .unwrap_or((640.0, 360.0));

    let mut rng = rand::rng();
    for i in 0..BRICK_COUNT {
        let x = rng.random_range(-half_width..half_width);
        let y = rng.random_range(-half_height..half_height);
        let brick_color = Color::srgb(
            rng.random_range(0.5..1.0),
            rng.random_range(0.5..1.0),
            rng.random_range(0.5..1.0),
        );
        brick(
            &mut commands,
            front.clone(),
            back.clone(),
            font.clone().into(),
            brick_color,
            Color::BLACK,
            'A',
            Transform::from_xyz(x, y, i as f32 * 0.1),
            FlipAngle {
                x: 0.0,
                y: PI,
                z: i as f32 * TAU / BRICK_COUNT as f32,
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
    let z_start = angle.z;

    let entity = commands
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
        )
        .id();

    // Continuously spins FlipAngle::z at a constant rate; since start and end are
    // exactly one full turn apart, the repeat loop is seamless.
    let target = entity.into_target();
    commands
        .animation()
        .repeat(Repeat::Infinitely)
        .insert(tween(
            Duration::from_secs_f32(TAU / Z_SPIN_SPEED),
            EaseKind::Linear,
            target.with(flip_angle_z(z_start, z_start + TAU)),
        ));
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
