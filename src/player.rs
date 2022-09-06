use bevy::utils::HashMap;
use bevy_inspector_egui::Inspectable;
use bevy_inspector_egui::InspectorPlugin;
use inline_tweak::*;

use bevy::ecs::system::Command;
use bevy::input::mouse::MouseButton;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::time::FixedTimestep;
use bevy_rapier3d::prelude::*;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default, Inspectable)]
struct InputState {
    pitch: f32,
    yaw: f32,
}

/// Mouse sensitivity and movement speed
#[derive(Inspectable)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub max_acceleration_force: f32,
    pub fov: f32,
}

#[derive(Default, Inspectable)]
struct PlayerState {
    grounded: bool,
    grabbing: Option<Entity>
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            max_speed: 10.,
            acceleration: 300.,
            max_acceleration_force: 200.,
            fov: 90.,
        }
    }
}

/// Used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FPSCam;

#[derive(Component)]
pub struct FPSBody;

#[derive(Component, Default)]
pub struct Grabbable {
    grabbed: bool
}

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
fn setup_player(
    mut commands: Commands,
    settings: Res<MovementSettings>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera = commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0., 0.95, 0.).looking_at(Vec3::ZERO, Vec3::Y),
            projection: PerspectiveProjection {
                fov: (settings.fov / 360.0) * (std::f32::consts::PI * 2.0),
                ..Default::default()
            }
            .into(),
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
        .insert(Friction {
            coefficient: 5.,
            combine_rule: CoefficientCombineRule::Average,
        })
        .insert(ExternalImpulse::default())
        .insert(ExternalForce::default())
        .insert(ColliderMassProperties::Density(2.0))
        .insert(FPSBody)
        .add_child(camera);

    let ground_size = 100.;
    let ground_height = 0.2;

 let texture_handle = assets.load("tex.jpg");

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: ground_size })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle.clone()),
                ..Default::default()
            }),
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        })
        .insert(Collider::cuboid(ground_size/2., ground_height, ground_size/2.));
}

fn detect_ground(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    mut state: ResMut<PlayerState>,
    query: Query<(Entity, &Transform), With<FPSBody>>,
) {
    for (entity, transform) in query.iter() {
        let ray_pos = transform.translation;
        let ray_dir = Vec3::new(0.0, -1., 0.0);
        let max_toi = 1.5;
        let solid = false;
        let groups = InteractionGroups::all();
        let filter = QueryFilter::default().exclude_rigid_body(entity);
        let mut grounded = false;
        if let Some((entity, toi)) =
            rapier_context.cast_ray(ray_pos, ray_dir, max_toi, solid, filter)
        {
            // The first collider hit has the entity `entity` and it hit after
            // the ray travelled a distance equal to `ray_dir * toi`.
            grounded = true;
        }
        state.grounded = grounded;
    }
}

fn player_grab(
    keys: Res<Input<KeyCode>>,
    mut state: ResMut<PlayerState>,
    mut commands: Commands,
    camera: Query<&GlobalTransform, With<FPSCam>>,
    mut grabbables: Query<(Entity, &mut Transform, &mut Velocity, &mut ExternalForce, &mut Grabbable, &mut GravityScale), With<Grabbable>>,
    rapier_context: Res<RapierContext>,
) {



    if !keys.just_pressed(KeyCode::E){
        if let Some(grabbedEnt) = state.grabbing {
            if let Ok((_, mut trans, mut vel, mut extforce, _, mut gscale)) = grabbables.get_mut(grabbedEnt) {

                for _global_transform in camera.iter() {
                    *vel = Velocity::zero();
                    let camtrans = _global_transform.compute_transform();
                    let grablocation = camtrans.translation + (camtrans.forward() * 1.5);
                    let direction =  (grablocation - trans.translation).normalize();
                    let distance = grablocation.distance(trans.translation);



                    if (direction != Vec3::ZERO && distance > 0.005) {
                    //extforce.force = direction * (distance.sqrt().powf(10.) + distance * 1000.);
                    let press = ((((distance + 0.03) * 3.).log10() + 1.) * distance.sqrt());
                        println!("{} {}", press, distance);
                    extforce.force = direction * ((press.abs() + press) / 2.) * 1500.;
                        //println!("{:?}", extforce.force);
                    } else {
                        extforce.force = Vec3::ZERO;
                    }
                    //trans.translation = camtrans.translation + (camtrans.forward() * 1.5);
                }
            }
        }
        return
    } else {

        let mut mapthing: HashMap<Entity, bool> = HashMap::new();

        for (ent, _, _, _, grabby, _) in grabbables.iter() {
            mapthing.insert(ent, grabby.grabbed);
        }

        if state.grabbing.is_none() {
            let pred = &|v| {
                if let Some(v) = mapthing.get(&v) {
                    return !v.to_owned()
                }
                false
            };
            let filter = QueryFilter::new().predicate(pred);

            for _global_transform in camera.iter() {
                let transform = _global_transform.compute_transform();
                let ray_pos = transform.translation;
                let ray_dir = transform.forward();
                let max_toi = 3.;
                let solid = false;


                if let Some((entity, toi)) =
                    rapier_context.cast_ray(ray_pos, ray_dir, max_toi, solid, filter)
                {
                    if let Ok((ent, transform, velocity, mut force, mut grabb, mut grav)) = grabbables.get_mut(entity) {
                        if(grabb.grabbed){
                            return;
                        }
                        state.grabbing = Some(ent);
                        grabb.grabbed = true;
                        grav.0 = 0.;

                        println!("GRABBED");
                    }

                    //let hit_point = ray_pos + ray_dir * toi;
                }
            }

        } else {
            if let Ok((ent, _, _, mut force,  mut grabby, mut grav)) = grabbables.get_mut(state.grabbing.unwrap()) {
                grabby.grabbed = false;
                state.grabbing = Option::None;

                force.force = Vec3::ZERO;
                println!("letgo");
                grav.0 = GravityScale::default().0;

            }
        }
    }

}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    state: Res<PlayerState>,
    cam_query: Query<&GlobalTransform, With<FPSCam>>,
    mut query: Query<(&mut ExternalForce, &Velocity), With<FPSBody>>,
) {
    let window = windows.get_primary().unwrap();
    for global_trans in cam_query.iter() {
        for (mut ext_impulse, body_velocity) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let mut y_velocity = 0.;
            let local_z = global_trans.back();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                if window.cursor_locked() {
                    match key {
                        KeyCode::W => velocity += forward,
                        KeyCode::S => velocity -= forward,
                        KeyCode::A => velocity -= right,
                        KeyCode::D => velocity += right,
                       _ => (),
                    }
                }
            }
                        if(keys.just_pressed(KeyCode::Space)) {
                            if state.grounded {
                                y_velocity += tweak!(1000.0)
                            }
                        }


            velocity = velocity.normalize();
            if (velocity == Vec3::ZERO || velocity.is_nan()) && y_velocity == 0. {
                ext_impulse.force = Vec3::ZERO;

                return;
            }
            velocity *= settings.max_speed;
            velocity.max((settings.acceleration * time.delta().as_secs_f32()) * Vec3::ONE);

            let needed_accel: Vec3 = (velocity - body_velocity.linvel) / time.delta().as_secs_f32();

            let newforce =
                needed_accel.clamp(needed_accel, settings.max_acceleration_force * Vec3::ONE);

            let mut composed_velocity = Vec3::ZERO;

            if !velocity.is_nan() {
                composed_velocity = Vec3::new(newforce.x, 0., newforce.z);
            }

            if y_velocity != 0. {
                composed_velocity.y = y_velocity;
            }

            println!("{:?}", velocity);

            if composed_velocity != Vec3::ZERO {
                ext_impulse.force = composed_velocity;
            }
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
        Query<&mut Transform, With<FPSBody>>,
    )>,
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
        let new_transform =
            Quat::from_axis_angle(Vec3::Y, 0.) * Quat::from_axis_angle(Vec3::X, state.pitch);

        let new_body_transform = Quat::from_axis_angle(Vec3::Y, state.yaw);

        for mut transform in set.p0().iter_mut() {
            if (transform.rotation != new_transform) {
                transform.rotation = new_transform;
            }
        }
        for mut transform in set.p1().iter_mut() {
            if (transform.rotation != new_body_transform) {
                transform.rotation = new_body_transform;
            }
        }
    }

    for mut bodytransform in set.p0().iter_mut() {}
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
            .init_resource::<PlayerState>()
            .add_startup_system(setup_player)
            .add_startup_system(initial_grab_cursor)
            .add_system_set(SystemSet::new().with_run_criteria(FixedTimestep::step(0.017)))
            .add_system(detect_ground)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab)
            .add_system(player_grab)
            .add_plugin(InspectorPlugin::<PlayerState>::new())
            .add_plugin(InspectorPlugin::<MovementSettings>::new())
            .add_plugin(InspectorPlugin::<InputState>::new());
    }
}
