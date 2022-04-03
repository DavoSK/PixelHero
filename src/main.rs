// ピクセルヒーロー

use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorPlugin, Inspectable, RegisterInspectable};

use bevy::{
    input::{keyboard::KeyCode, Input},
};

#[derive(Component, Inspectable)]
struct Player {
    health: f32,
    speed: f32,
    animation_state: i32,
    is_local: bool
}

#[derive(Component, Inspectable)]
struct Movement {
    dir: Vec2,
    velocity: Vec2
}

fn create_player() -> Player {
    Player { health: 100.0, speed: 2.0, animation_state: 0, is_local: true }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Movement>()
        .register_inspectable::<Player>()
        .add_startup_system(setup)
        .add_system(animate_sprite_system)
        .add_system(local_player_input_system)
        .add_system(entity_movement_system)
        .run();
}

fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

fn entity_movement_system(time: Res<Time>, mut query: Query<(&mut Transform, &Movement)>) {
    let dt = time.delta().as_secs_f32() * 1000.0;
    for (mut transform, movement) in query.iter_mut() {
        transform.translation.x += dt * movement.dir.x * movement.velocity.x;
        transform.translation.y += dt * movement.dir.y * movement.velocity.y;
    }
}

fn local_player_input_system(keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&mut Movement, &Player)>) {
    for (mut movement, player) in query.iter_mut() {
        if !player.is_local { continue };
        movement.dir.x = if keyboard_input.pressed(KeyCode::A) { -1.0 } else if keyboard_input.pressed(KeyCode::D) { 1.0 } else { 0.0 };
        movement.dir.y = if keyboard_input.pressed(KeyCode::W) { 1.0 } else if keyboard_input.pressed(KeyCode::S) { -1.0 } else { 0.0 };
        movement.velocity = Vec2::new(player.speed, player.speed);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/gabe-idle-run.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true))
        .insert(Movement{ dir: Vec2::default(), velocity: Vec2::default()})
        .insert(create_player());
}