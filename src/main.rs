mod player;
use bevy::prelude::*;
use player::*;

use std::f32::consts::PI;

const FULL_TURN: f32 = 2.0 * PI;

// Define a component to designate a rotation speed to an entity.
#[derive(Component)]
struct Rotatable {
    speed: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 9.0,
            fov: 110.// default: 12.0
        })
        .add_system(rotate_cube)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a cube to rotate.
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 10.0 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        })
        .insert(Rotatable { speed: 0.3 });

    // Add a light source for better 3d visibility.
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::ONE * 11.0),
        ..Default::default()
    });
}

// This system will rotate any entity in the scene with an assigned Rotatable around its z-axis.
fn rotate_cube(mut cubes: Query<(&mut Transform, &Rotatable)>, timer: Res<Time>) {
    for (mut transform, cube) in cubes.iter_mut() {
        // The speed is taken as a percentage of a full 360 degree turn.
        // The timers delta_seconds is used to smooth out the movement.
        let rotation_change = Quat::from_rotation_y(FULL_TURN * cube.speed * timer.delta_seconds());
        transform.rotate(rotation_change);
    }
}
