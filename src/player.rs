use std::arch::x86_64::_pext_u32;

use bevy::ecs::system::Command;
use bevy::time::FixedTimestep;
use bevy::input::mouse::MouseButton;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::Damping;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::ExternalImpulse;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::RigidBody;
use bevy_rapier3d::prelude::Velocity;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default)]
struct InputState {
    pitch: f32,
    yaw: f32,
}

/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub max_acceleration_force: f32,
    pub fov: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            max_speed: 8.,
            acceleration: 200.,
            max_acceleration_force: 150.,
            fov: 90.,
        }
    }
}

/// Used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FPSCam;


#[derive(Component)]
pub struct FPSBody;

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
fn setup_player(mut commands: Commands, settings: Res<MovementSettings>
    ,mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>
) {
    let camera = commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0., 0.95, 0.).looking_at(Vec3::ZERO, Vec3::Y),
            projection: PerspectiveProjection {
                fov: (settings.fov / 360.0) * (std::f32::consts::PI * 2.0),
                ..Default::default()
            }.into(),
            ..Default::default()
        })
        .insert(FPSCam)
        .id();

        commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(-2.0, 10.0, 5.0)))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::capsule_y(1., 0.5))
        .insert(Velocity::default())
        .insert(Damping {
            linear_damping: 2.0,
            ..Default::default()
        })
        .insert(ExternalImpulse::default())
        .insert(ExternalForce::default())
        .insert(FPSBody)
        .add_child(camera);

    let ground_size = 200.1;
    let ground_height = 0.1;

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 200.1 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        })
        .insert(Collider::cuboid(ground_size, ground_height, ground_size));
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<( &Transform, &mut ExternalForce, &Velocity), With<FPSBody>>,
) {
    let window = windows.get_primary().unwrap();
    for (transform, mut ext_impulse, body_velocity) in query.iter_mut() {
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
        if (velocity == Vec3::ZERO || velocity.is_nan()) {
            ext_impulse.force = Vec3::ZERO;
            //TODO: ADD CUSTOM DAMPENING

            return;
        }
        velocity *= settings.max_speed;
        velocity.max((settings.acceleration * time.delta().as_secs_f32()) * Vec3::ONE);

        let needed_accel: Vec3 = (velocity - body_velocity.linvel) / time.delta().as_secs_f32();

        let newforce = needed_accel.clamp(needed_accel, settings.max_acceleration_force * Vec3::ONE);




        if !velocity.is_nan() {
            ext_impulse.force = Vec3::new(newforce.x, 0., newforce.z);
        }
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    mut mouse_move: EventReader<MouseMotion>,
    mut set: ParamSet<(
        Query<&mut Transform, With<FPSCam>>,
        Query<&mut Transform, With<FPSBody>>
    )>
) {
    let window = windows.get_primary().unwrap();

        for ev in mouse_move.iter() {
            if window.cursor_locked() {
                // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                let window_scale = window.height().min(window.width());
                state.pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                state.yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
            }

            state.pitch = state.pitch.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            let new_transform = Quat::from_axis_angle(Vec3::Y, 0.)
                * Quat::from_axis_angle(Vec3::X, state.pitch);

            let new_body_transform = Quat::from_axis_angle(Vec3::Y, state.yaw);


            for mut transform in set.p0().iter_mut() {
                if (transform.rotation != new_transform) {
                    transform.rotation = new_transform;
                }
            }
            for mut transform in set.p1().iter_mut() {
                if (transform.rotation != new_body_transform){
                transform.rotation = new_body_transform;
            }
            }

        }


    for mut bodytransform in set.p0().iter_mut() {

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
