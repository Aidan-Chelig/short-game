mod player;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::MassProperties;
use inline_tweak::*;
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
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        //.add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .insert_resource(MovementSettings::default())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        ..default()
    });

    let cube_handle = asset_server.load("food_apple_01_4k.glb#Scene0");
    let croissant_handle = asset_server.load("croissant.glb#Scene0");

    commands
        .spawn_bundle(SceneBundle {
            scene: cube_handle,
            transform: Transform::from_xyz(0., 10., 0.).with_scale(Vec3::ONE),
            ..Default::default()
        }) // Spawn a cube to rotate.
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Velocity::default())
        .insert(ExternalForce::default())
        .insert(GravityScale::default())
        .insert(ColliderMassProperties::Density(5.0))
        .insert(Grabbable::default())
        .insert(Damping {
            linear_damping: 1.,
            ..Default::default()
        });

    commands
        .spawn_bundle(SceneBundle {
            scene: croissant_handle,
            transform: Transform::from_xyz(0., 10., 0.).with_scale(Vec3::ONE),
            ..Default::default()
        }) // Spawn a cube to rotate.
        .insert(RigidBody::Dynamic)
        .insert(Collider::capsule_x(0.4, 0.4 / 2.))
        .insert(Velocity::default())
        .insert(ExternalForce::default())
        .insert(ColliderMassProperties::Density(5.0))
        .insert(GravityScale::default())
        .insert(Grabbable::default())
        .insert(Damping {
            linear_damping: 1.,
            ..Default::default()
        });

    //
    // Add a light source for better 3d visibility.
    //
    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
    //
}

// This system will rotate any entity in the scene with an assigned Rotatable around its z-axis.
