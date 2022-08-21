use bevy::time::FixedTimestep;
use bevy::input::mouse::MouseButton;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default)]
struct InputState {
    pitch: f32,
    yaw: f32,
}

/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
    pub fov: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
            fov: 90.,
        }
    }
}

/// Used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    toggle_grab_cursor(windows.get_primary_mut().unwrap());
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands, settings: Res<MovementSettings>) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: PerspectiveProjection {
                fov: (settings.fov / 360.0) * (std::f32::consts::PI * 2.0),
                ..Default::default()
            }.into(),
            ..Default::default()
        })
        .insert(FlyCam);
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    for (_camera, mut transform) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);

        for key in keys.get_pressed() {
            if window.cursor_locked() {
                match key {
                    KeyCode::W => velocity += forward,
                    KeyCode::S => velocity -= forward,
                    KeyCode::A => velocity -= right,
                    KeyCode::D => velocity += right,
                    KeyCode::Space => velocity += Vec3::Y,
                    KeyCode::LShift => velocity -= Vec3::Y,
                    _ => (),
                }
            }
        }

        velocity = velocity.normalize();

        if !velocity.is_nan() {
            transform.translation += velocity * time.delta_seconds() * settings.speed
        }
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    mut mouse_move: EventReader<MouseMotion>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    for (_camera, mut transform) in query.iter_mut() {
        for ev in mouse_move.iter() {
            if window.cursor_locked() {
                // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                let window_scale = window.height().min(window.width());
                state.pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                state.yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
            }

            state.pitch = state.pitch.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            let new_transform = Quat::from_axis_angle(Vec3::Y, state.yaw)
                * Quat::from_axis_angle(Vec3::X, state.pitch);
            if transform.rotation != new_transform {
                transform.rotation = new_transform;
            }
        }
    }
}

fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    mut mouse_click: Res<Input<MouseButton>>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();

    //window.set_cursor_lock_mode(!window.cursor_locked());
    //window.set_cursor_visibility(!window.cursor_visible());

    if window.cursor_locked() && !window.cursor_visible() && keys.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    } else if mouse_click.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }
}

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_startup_system(setup_player)
            .add_startup_system(initial_grab_cursor)
            .add_system_set(SystemSet::new().with_run_criteria(FixedTimestep::step(0.017)))
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab);
    }
}
