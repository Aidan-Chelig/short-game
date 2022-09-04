mod player;
use bevy::prelude::*;
use inline_tweak::*;
use player::*;
use bevy_rapier3d::prelude::*;
use bevy_inspector_egui::{InspectorPlugin, Inspectable};
use bevy_inspector_egui::WorldInspectorPlugin;

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
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
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

        commands.spawn_bundle(SceneBundle {
            scene: cube_handle,
            transform: Transform::from_xyz(0.,10., 0. ).with_scale(Vec3::ONE),
            ..Default::default()

        })    // Spawn a cube to rotate.
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Velocity::default())
        .insert(ExternalForce::default())
        .insert(Grabbable::default());
    //
    // Add a light source for better 3d visibility.
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0., 10., 0.)),
        ..Default::default()
    });
}

// This system will rotate any entity in the scene with an assigned Rotatable around its z-axis.
