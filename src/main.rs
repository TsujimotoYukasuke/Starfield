// Copyright Quentin Wright 2022, All Rights Reserved.

use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::Rng;

#[cfg(debug_assertions)]
use bevy_inspector_egui::WorldInspectorPlugin;

const ACCELERATION_MULTIPLIER: f32 = 1.0;
const SPACE_EXTENT: f32 = 1000.0;
const MOVE_SPEED_RANGE: RangeInclusive<f32> = 10.0..=80.0;
const NUM_STARS: u32 = 1300;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugPlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PreUpdate, reset_stars)
        .add_system_to_stage(CoreStage::Update, calculate_velocity)
        .add_system_to_stage(CoreStage::PostUpdate, move_stars)
        .run();
}

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    #[cfg(debug_assertions)]
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .register_type::<Star>();
    }

    #[cfg(not(debug_assertions))]
    fn build(&self, _: &mut App) {}
}

#[derive(Reflect, Component)]
#[reflect(Component)]
struct Star {
    velocity: Vec3,
    base_speed: f32,
}

impl Default for Star {
    fn default() -> Self {
        let base_speed = rand_in_range(MOVE_SPEED_RANGE);
        let velocity = Vec3::default();

        Self {
            velocity,
            base_speed,
        }
    }
}

#[derive(Component)]
struct StarTrail;

/// Sets up the starfield.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera.
    commands.spawn_bundle(Camera2dBundle::default());

    for _ in 0..=NUM_STARS {
        // Random (x, y) position.
        let x = rand_in_range(space_extent());
        let y = rand_in_range(space_extent());
        let z = rand_in_range(half_space_extent());
        let transform = Transform::from_translation(Vec3::new(x, y, z));

        // Spawn the star.
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(1.0).into()).into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                transform,
                ..default()
            })
            .insert(Star::default());
    }
}

/// Moves the stars based on their current velocity.
fn move_stars(time: Res<Time>, mut query: Query<(&Star, &mut Transform)>) {
    for (star, mut transform) in query.iter_mut() {
        transform.translation += star.velocity * time.delta_seconds();
    }
}

/// Calculates velocity based on the speed of the star as well as the current acceleration.
fn calculate_velocity(time: Res<Time>, mut query: Query<(&mut Star, &Transform)>) {
    for (mut star, transform) in query.iter_mut() {
        // We're dealing with 2D so we want to disregard the z dimension which will be used for parallax.
        let xy_coords = Vec3::new(transform.translation.x, transform.translation.y, 0.0);

        // We're always moving away from the origin, so we don't have to calculate direction.
        let movement_direction = xy_coords.normalize();

        // Acceleration scaled with distance.
        // We only multiply delta once even though a = s*(dt^2) this is because we'll multiply velocity later.
        let acceleration = xy_coords.length() * ACCELERATION_MULTIPLIER * time.delta_seconds();
        let velocity = movement_direction * acceleration * star.base_speed;

        star.velocity = velocity;
    }
}

/// Takes stars outside the space extent and places them back inside.
fn reset_stars(mut query: Query<(&mut Star, &mut Transform)>) {
    // Checks if a location is outside of the space extent.
    let outside_extent = |t: Vec3| !space_extent().contains(&t.x) || !space_extent().contains(&t.y);

    query
        .iter_mut()
        .filter(|(_, transform)| outside_extent(transform.translation))
        .for_each(|(mut star, mut transform)| {
            let x = rand_in_range(half_space_extent());
            let y = rand_in_range(half_space_extent());
            let z = rand_in_range(half_space_extent());

            transform.translation = Vec3::new(x, y, z);
            star.base_speed = rand_in_range(MOVE_SPEED_RANGE);
        });
}

/// Generates a random value within a range.
fn rand_in_range<T, R>(range: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    // This function is really just a short way of doing this.
    rand::thread_rng().gen_range(range)
}

/// Turns the SPACE_EXTENT constant into a range.
fn space_extent() -> RangeInclusive<f32> {
    -SPACE_EXTENT..=SPACE_EXTENT
}

/// Turns the SPACE_EXTENT constant into a range, half the size of the entire extent.
fn half_space_extent() -> RangeInclusive<f32> {
    let half_extent = SPACE_EXTENT / 2.0;
    -half_extent..=half_extent
}
