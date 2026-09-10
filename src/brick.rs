use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tween::{
    combinator::{parallel, tween},
    prelude::*,
};
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

/// [`Interpolator`] for [`FlipAngle::x`].
#[derive(Debug, Clone, Copy)]
pub struct FlipAngleX {
    pub start: f32,
    pub end: f32,
}

impl Interpolator for FlipAngleX {
    type Item = FlipAngle;

    fn interpolate(&self, item: &mut Self::Item, value: f32, _previous_value: f32) {
        item.x = self.start.lerp(self.end, value);
    }
}

/// Constructor for [`FlipAngleX`]
pub fn flip_angle_x(start: f32, end: f32) -> FlipAngleX {
    FlipAngleX { start, end }
}

/// [`Interpolator`] for [`FlipAngle::y`].
#[derive(Debug, Clone, Copy)]
pub struct FlipAngleY {
    pub start: f32,
    pub end: f32,
}

impl Interpolator for FlipAngleY {
    type Item = FlipAngle;

    fn interpolate(&self, item: &mut Self::Item, value: f32, _previous_value: f32) {
        item.y = self.start.lerp(self.end, value);
    }
}

/// Constructor for [`FlipAngleY`]
pub fn flip_angle_y(start: f32, end: f32) -> FlipAngleY {
    FlipAngleY { start, end }
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
pub struct DragTracker {
    pub was_dragged: bool,
    /// True while the pointer is holding and dragging this brick.
    dragging: bool,
    /// The brick's velocity before the drag began, restored on release.
    saved_velocity: Option<Vec2>,
    /// Whether the idle spin was running when the drag began.
    had_spin: bool,
    /// The brick's rigid body type before the drag began, restored on release.
    saved_rigid_body: Option<RigidBody>,
    /// The brick's position when the drag began; restored if the brick is
    /// dropped outside the walls.
    saved_translation: Vec3,
}

/// Inner-edge boundaries of the static walls that bound the play area.
#[derive(Resource)]
pub struct PlayAreaBounds {
    /// The left/right walls' inner edges sit at `x = ±half_width`.
    pub half_width: f32,
    /// The top wall's inner edge sits at `y = top`.
    pub top: f32,
    /// The bottom wall's inner edge sits at `y = bottom` (a negative value).
    pub bottom: f32,
}

/// A single drop slot: a [`SLOT_SIZE`] × [`SLOT_SIZE`] box centered at
/// `position`. The brick occupying it, if any, is recorded in `occupied`.
#[derive(Resource)]
pub struct Slot {
    /// The slot's center.
    pub position: Vec3,
    /// The brick currently occupying the slot, if any.
    pub occupied: Option<Entity>,
}

/// Tracks the entity driving a brick's idle continuous spin, so it can be
/// stopped before it fights the click-triggered flip animation.
#[derive(Component)]
pub struct SpinAnimation(Entity);

/// A brick dropped outside the walls, animating back to its pre-drag position.
#[derive(Component)]
pub struct ReturnAnimation {
    /// The brick's position at the moment of the out-of-bounds drop.
    from: Vec3,
    /// The brick's position when the drag began (the return target).
    to: Vec3,
    /// Seconds elapsed since the return animation started.
    elapsed: f32,
}

/// How long the click-triggered flip animation takes.
const FLIP_DURATION: Duration = Duration::from_millis(900);

/// How long a brick dropped outside the walls takes to animate back to its
/// pre-drag position.
const RETURN_DURATION: Duration = Duration::from_millis(500);

/// Side length, in pixels, of a brick's square collision box.
const BRICK_SIZE: f32 = 128.0;

/// Thickness of the invisible static walls bounding the play area.
const WALL_THICKNESS: f32 = 50.0;

/// Number of bricks spawned at random positions within the window.
const BRICK_COUNT: usize = 10;

/// Fraction of the window kept clear on each side as a spawn/collision margin.
const BORDER_MARGIN_FRACTION: f32 = 0.1;

/// Gap between the bottom wall and the bottom of the window, as a fraction of
/// the full window height (kept larger than the side/top margins).
const BOTTOM_MARGIN_FRACTION: f32 = 0.3;

/// Maximum speed, in pixels/second, a brick can spawn with.
const MAX_SPEED: f32 = 100.0;

/// Side length, in pixels, of the drop slot's square drop area.
const SLOT_SIZE: f32 = 100.0;

/// Half the slot size, used as the drop-area radius from the slot center.
const SLOT_HALF: f32 = SLOT_SIZE / 2.0;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    let front = asset_server.load("sprites/Button.png");
    let back = asset_server.load("sprites/Button.png");
    let font = asset_server.load("fonts/JameGem08_2026-Regular.ttf");

    let (half_width, half_height) = windows
        .single()
        .map(|window| (window.width() / 2.0, window.height() / 2.0))
        .unwrap_or((640.0, 360.0));
    let border_x = half_width * (1.0 - BORDER_MARGIN_FRACTION);
    let top_border = half_height * (1.0 - BORDER_MARGIN_FRACTION);
    let bottom_border = half_height * (1.0 - 2.0 * BOTTOM_MARGIN_FRACTION);
    spawn_walls(&mut commands, border_x, top_border, bottom_border);
    commands.insert_resource(PlayAreaBounds {
        half_width: border_x,
        top: top_border,
        bottom: -bottom_border,
    });

    // Bottom slots strip: centered (in Y) between the window bottom and the
    // bottom wall's inner edge, and offset from the left edge by the same side
    // margin as the walls.
    let slots = asset_server.load("sprites/slot.png");
    let slots_y = (-half_height + -bottom_border) / 2.0;
    let slots_x = -border_x;
    let slot_position = Vec3::new(slots_x, slots_y, -1.0);
    commands.insert_resource(Slot {
        position: slot_position,
        occupied: None,
    });
    commands.spawn((
        Sprite {
            image: slots,
            custom_size: Some(Vec2::new(SLOT_SIZE, SLOT_SIZE)),
            ..default()
        },
        Transform::from_xyz(slots_x, slots_y, -1.0),
    ));

    let mut rng = rand::rng();
    for i in 0..BRICK_COUNT {
        let x = rng.random_range(-border_x..border_x);
        let y = rng.random_range(-bottom_border..top_border);
        let brick_color = Color::srgb(
            rng.random_range(0.5..1.0),
            rng.random_range(0.5..1.0),
            rng.random_range(0.5..1.0),
        );
        let angle_rad = rng.random_range(0.0..TAU);
        let speed = rng.random_range(0.0..MAX_SPEED);
        let velocity = Vec2::new(angle_rad.cos(), angle_rad.sin()) * speed;
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
            velocity,
        );
    }
}

// Static colliders surrounding the play area so bricks bounce back inward.
// The top and bottom borders may differ (e.g. extra room at the bottom); the
// side walls span the full height between them and are re-centered to match.
fn spawn_walls(commands: &mut Commands, border_x: f32, top_border: f32, bottom_border: f32) {
    let wall = (RigidBody::Static, Restitution::new(1.0), Friction::ZERO);
    let side_height = top_border + bottom_border + WALL_THICKNESS * 2.0;
    let side_center_y = (top_border - bottom_border) / 2.0;
    commands.spawn((
        wall.clone(),
        Collider::rectangle(WALL_THICKNESS, side_height),
        Transform::from_xyz(-border_x - WALL_THICKNESS / 2.0, side_center_y, 0.0),
    ));
    commands.spawn((
        wall.clone(),
        Collider::rectangle(WALL_THICKNESS, side_height),
        Transform::from_xyz(border_x + WALL_THICKNESS / 2.0, side_center_y, 0.0),
    ));
    commands.spawn((
        wall.clone(),
        Collider::rectangle(border_x * 2.0 + WALL_THICKNESS * 2.0, WALL_THICKNESS),
        Transform::from_xyz(0.0, top_border + WALL_THICKNESS / 2.0, 0.0),
    ));
    commands.spawn((
        wall,
        Collider::rectangle(border_x * 2.0 + WALL_THICKNESS * 2.0, WALL_THICKNESS),
        Transform::from_xyz(0.0, -bottom_border - WALL_THICKNESS / 2.0, 0.0),
    ));
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
    velocity: Vec2,
) {
    let z_start = angle.z;

    let entity = commands
        .spawn((
            Brick,
            angle,
            DragTracker::default(),
            Visibility::default(),
            transform,
            RigidBody::Dynamic,
            Collider::rectangle(BRICK_SIZE, BRICK_SIZE),
            LinearVelocity(velocity),
            Restitution::new(1.0),
            Friction::ZERO,
            LockedAxes::ROTATION_LOCKED,
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
             mut query_tracker: Query<&mut DragTracker>,
             query_rb: Query<&RigidBody>,
             mut query_vel: Query<&mut LinearVelocity>,
             query_parent: Query<&ChildOf>,
             query_spin: Query<&SpinAnimation>,
             mut slot: ResMut<Slot>,
             mut commands: Commands| {
                // Act on the brick itself (it owns the collider and velocity) rather
                // than the pickable child face the pointer is on.
                let target = drag.event_target();
                let brick = query_parent
                    .get(target)
                    .map(|parent| parent.0)
                    .unwrap_or(target);
                if let Ok(mut tracker) = query_tracker.get_mut(brick) {
                    tracker.was_dragged = true;
                    if !tracker.dragging {
                        tracker.dragging = true;
                        // If this brick owns the slot, release it: the brick is
                        // being moved, so the slot reads as free until a brick
                        // drops in again.
                        if slot.occupied == Some(brick) {
                            slot.occupied = None;
                        }
                        // A return animation from a previous out-of-bounds drop
                        // is superseded by this new drag.
                        commands.entity(brick).remove::<ReturnAnimation>();
                        // Remember where the brick started so a drop outside the
                        // walls can send it back here.
                        if let Ok(transform) = query_transform.get(brick) {
                            tracker.saved_translation = transform.translation;
                        }
                        // Freeze the brick's movement for the duration of the drag,
                        // keeping any velocity already saved for a pending return.
                        if let Ok(mut vel) = query_vel.get_mut(brick) {
                            if tracker.saved_velocity.is_none() {
                                tracker.saved_velocity = Some(vel.0);
                            }
                            vel.0 = Vec2::ZERO;
                        }
                        // Make the brick kinematic so it stays in the collision
                        // system (shoving other bricks out of the way) but can't be
                        // pushed or gain velocity from collisions itself.
                        if let Ok(rb) = query_rb.get(brick) {
                            if tracker.saved_rigid_body.is_none() {
                                tracker.saved_rigid_body = Some(*rb);
                            }
                        }
                        commands.entity(brick).insert(RigidBody::Kinematic);
                        // Stop the idle spin so the brick holds still while positioned.
                        if query_spin.get(brick).is_ok() {
                            tracker.had_spin = true;
                        }
                        if let Ok(spin) = query_spin.get(brick) {
                            commands.entity(spin.0).despawn();
                        }
                        commands.entity(brick).remove::<SpinAnimation>();
                    }
                }
                if let Ok(mut transform) = query_transform.get_mut(brick) {
                    transform.translation.x += drag.delta.x;
                    transform.translation.y -= drag.delta.y;
                };
                drag.propagate(false);
            },
        )
        .observe(
            |mut click: On<Pointer<Release>>,
             mut commands: Commands,
             mut query_tracker: Query<&mut DragTracker>,
             mut query_transform: Query<&mut Transform>,
             bounds: Res<PlayAreaBounds>,
             query_angle: Query<&FlipAngle>,
             query_parent: Query<&ChildOf>,
             query_spin: Query<&SpinAnimation>,
             mut slot: ResMut<Slot>,
             mut query_vel: Query<&mut LinearVelocity>| {
                // Read event details
                println!(
                    "Entity {:?} was released by pointer {:?}",
                    click.event_target(),
                    click.pointer_id
                );

                let target = click.event_target();
                let brick = query_parent
                    .get(target)
                    .map(|parent| parent.0)
                    .unwrap_or(target);
                if let Ok(mut tracker) = query_tracker.get_mut(brick) {
                    if tracker.dragging {
                        let t = if let Ok(transform) = query_transform.get(brick) {
                            transform.translation
                        } else {
                            Vec3::ZERO
                        };
                        let half = BRICK_SIZE / 2.0;
                        let outside = t.x - half < -bounds.half_width
                            || t.x + half > bounds.half_width
                            || t.y + half > bounds.top
                            || t.y - half < bounds.bottom;
                        // A drop over the slot is valid even though the slot sits
                        // outside the walls (below the bottom border), so it is
                        // tested before the out-of-bounds check.
                        let on_slot = (t.x - slot.position.x).abs() <= SLOT_HALF
                            && (t.y - slot.position.y).abs() <= SLOT_HALF;
                        // The slot is free for this brick if it's empty or if this
                        // brick is the one currently occupying it (re-dropped).
                        let can_park = match slot.occupied {
                            Some(occupied) => occupied == brick,
                            None => true,
                        };
                        if on_slot {
                            if can_park {
                                // Snap into the slot and freeze: a static body
                                // keeps colliding but can't move; zero velocity,
                                // no idle spin, and settle flat at angle 0.
                                if let Ok(mut transform) = query_transform.get_mut(brick) {
                                    transform.translation = slot.position;
                                }
                                commands.entity(brick).insert(RigidBody::Static);
                                if let Ok(mut vel) = query_vel.get_mut(brick) {
                                    vel.0 = Vec2::ZERO;
                                }
                                if let Ok(spin) = query_spin.get(brick) {
                                    commands.entity(spin.0).despawn();
                                }
                                commands.entity(brick).remove::<SpinAnimation>();
                                tracker.had_spin = false;
                                slot.occupied = Some(brick);
                                if let Ok(angle) = query_angle.get(brick) {
                                    settle_brick(&mut commands, brick, &angle);
                                }
                            } else {
                                // Occupied by a different brick: the occupant
                                // stays; the incoming brick returns to where the
                                // drag began (see return_animation_system).
                                commands.entity(brick).insert(ReturnAnimation {
                                    from: t,
                                    to: tracker.saved_translation,
                                    elapsed: 0.0,
                                });
                            }
                        } else if outside {
                            // A drop outside the walls isn't valid: the brick will
                            // animate back to where it was when the drag started.
                            commands.entity(brick).insert(ReturnAnimation {
                                from: t,
                                to: tracker.saved_translation,
                                elapsed: 0.0,
                            });
                            // The brick stays kinematic with zero velocity until
                            // the return animation completes; that also resumes
                            // its saved velocity and idle spin (see
                            // return_animation_system).
                        } else {
                            // Drag ended: put the brick back under the physics
                            // simulation's control, then resume its movement and,
                            // if it was running before the drag, its idle spin.
                            if let Some(rigid_body) = tracker.saved_rigid_body.take() {
                                commands.entity(brick).insert(rigid_body);
                            }
                            if let Some(velocity) = tracker.saved_velocity.take() {
                                if let Ok(mut vel) = query_vel.get_mut(brick) {
                                    vel.0 = velocity;
                                }
                            }
                            if tracker.had_spin && query_spin.get(brick).is_err() {
                                if let Ok(angle) = query_angle.get(brick) {
                                    spawn_spin_animation(&mut commands, brick, angle.z);
                                }
                            }
                            tracker.had_spin = false;
                        }
                        tracker.dragging = false;
                    } else if !tracker.was_dragged {
                        println!("Click.");
                        if let Ok(angle) = query_angle.get(brick) {
                            // Stop the idle spin so it doesn't fight the flip.
                            if let Ok(spin) = query_spin.get(brick) {
                                commands.entity(spin.0).despawn();
                            }
                            commands.entity(brick).remove::<SpinAnimation>();
                            flip_brick(&mut commands, brick, &angle);
                        }
                    }
                    tracker.was_dragged = false;
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
    spawn_spin_animation(commands, entity, z_start);
}

/// Spawns the idle continuous-spin animation for `brick` and records its
/// animation entity on the brick so it can be stopped (and later re-spawned).
fn spawn_spin_animation(commands: &mut Commands, brick: Entity, z_start: f32) {
    let target = brick.into_target();
    let spin_entity = commands
        .animation()
        .repeat(Repeat::Infinitely)
        .insert(tween(
            Duration::from_secs_f32(TAU / Z_SPIN_SPEED),
            EaseKind::Linear,
            target.with(flip_angle_z(z_start, z_start + TAU)),
        ))
        .id();
    commands.entity(brick).insert(SpinAnimation(spin_entity));
}

// Rotates x by 2 full turns and y by 1 full turn, landing on whichever face
// (front/back) isn't currently showing, plus a 180° roll on z for flair.
fn flip_brick(commands: &mut Commands, entity: Entity, angle: &FlipAngle) {
    let x_start = angle.x;
    let y_start = angle.y;
    let z_start = angle.z;

    // Mirrors flip_faces_system's facing check to decide which way to land.
    let currently_facing_camera = angle.y.cos() * angle.x.cos() > 0.0;
    let y_landing_phase = if currently_facing_camera { PI } else { 0.0 };

    let x_end = x_start + (0.0 - x_start).rem_euclid(TAU) + 2.0 * TAU;
    let y_end = y_start + (y_landing_phase - y_start).rem_euclid(TAU) + TAU;
    let z_end = z_start + PI;

    let target = entity.into_target();
    commands.animation().insert(parallel((
        tween(
            FLIP_DURATION,
            EaseKind::QuadraticInOut,
            target.with(flip_angle_x(x_start, x_end)),
        ),
        tween(
            FLIP_DURATION,
            EaseKind::QuadraticInOut,
            target.with(flip_angle_y(y_start, y_end)),
        ),
        tween(
            FLIP_DURATION,
            EaseKind::QuadraticInOut,
            target.with(flip_angle_z(z_start, z_end)),
        ),
    )));
}

/// Settle a parked brick flat and front-facing ("angle 0"): rotate each axis to
/// the nearest full turn so [`flip_faces_system`] renders it unsquished, front
/// face showing, with zero roll.
fn settle_brick(commands: &mut Commands, entity: Entity, angle: &FlipAngle) {
    let x_start = angle.x;
    let y_start = angle.y;
    let z_start = angle.z;

    let x_end = x_start + (0.0 - x_start).rem_euclid(TAU);
    let y_end = y_start + (0.0 - y_start).rem_euclid(TAU);
    let z_end = z_start + (0.0 - z_start).rem_euclid(TAU);

    let target = entity.into_target();
    commands.animation().insert(parallel((
        tween(
            FLIP_DURATION,
            EaseKind::QuadraticInOut,
            target.with(flip_angle_x(x_start, x_end)),
        ),
        tween(
            FLIP_DURATION,
            EaseKind::QuadraticInOut,
            target.with(flip_angle_y(y_start, y_end)),
        ),
        tween(
            FLIP_DURATION,
            EaseKind::QuadraticInOut,
            target.with(flip_angle_z(z_start, z_end)),
        ),
    )));
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

/// Smoothly animates a brick that was dropped outside the walls back to its
/// pre-drag position over [`RETURN_DURATION`], then hands it back to the
/// physics simulation: restoring its saved velocity, rigid body and (if it was
/// running before the drag) its idle spin.
pub fn return_animation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut animations: Query<(Entity, &mut ReturnAnimation, &mut Transform)>,
    mut trackers: Query<&mut DragTracker>,
    query_spin: Query<&SpinAnimation>,
    query_angle: Query<&FlipAngle>,
    mut query_vel: Query<&mut LinearVelocity>,
) {
    for (brick, mut ret, mut transform) in &mut animations {
        ret.elapsed += time.delta_secs();
        let t = (ret.elapsed / RETURN_DURATION.as_secs_f32()).min(1.0);
        // Ease in and out so the return accelerates away from and settles into
        // the original position.
        let eased = t * t * (3.0 - 2.0 * t);
        transform.translation = ret.from.lerp(ret.to, eased);
        if t >= 1.0 {
            if let Ok(mut tracker) = trackers.get_mut(brick) {
                if let Some(rigid_body) = tracker.saved_rigid_body.take() {
                    commands.entity(brick).insert(rigid_body);
                }
                if let Some(velocity) = tracker.saved_velocity.take() {
                    if let Ok(mut vel) = query_vel.get_mut(brick) {
                        vel.0 = velocity;
                    }
                }
                if tracker.had_spin && query_spin.get(brick).is_err() {
                    if let Ok(angle) = query_angle.get(brick) {
                        spawn_spin_animation(&mut commands, brick, angle.z);
                    }
                }
                tracker.had_spin = false;
            }
            commands.entity(brick).remove::<ReturnAnimation>();
        }
    }
}
