// ピクセルヒーロー
use serde::{Deserialize, Serialize};
use serde_json::Result;

use bevy::prelude::*;
use bevy_inspector_egui::{WorldInspectorPlugin, Inspectable, RegisterInspectable};

use bevy::{
    input::{keyboard::KeyCode, Input},
};

#[derive(Component, Inspectable)]
struct Player {
    health: f32,
    speed: f32,
    animation_state: u32,
    is_local: bool
}

fn is_animation_idle(state: u32) -> bool {
    state > 3
}

#[derive(Deserialize)]
struct AnimationFrame {
    name: String,
    duration: f32
}

#[derive(Component, Deserialize)]
struct Animation {
    texture_atlas: String,
    row_cnt: i32,
    tile_size: u32,
    frames: Vec<AnimationFrame>,
    current_col_idx: i32,
    last_frame_time: f32
}

enum AnimationState {
    FrontLeft,
    FrontRight,
    BackLeft,
    BackRight,
    IdleFrontLeft,
    IdleFrontRight,
    IdleBackLeft,
    IdleBackRight
}

#[derive(Component, Inspectable)]
struct Movement {
    dir: Vec2,
    last_moving_dir: Vec2,
    velocity: Vec2,
}

fn create_player() -> Player {
    Player { health: 100.0, speed: 0.5, animation_state: 0, is_local: true }
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
        .add_system(entity_movement_animator_system)
        .run();
}

fn animate_sprite_system(
    time: Res<Time>,
    mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>
) {
    for (mut sprite, mut animation) in query.iter_mut() {
        animation.last_frame_time = time.delta_seconds();
        let current_frame = &animation.frames[animation.current_col_idx as usize];
        if animation.last_frame_time > current_frame.duration {
            animation.last_frame_time = 0.0;

            animation.current_col_idx = animation.current_col_idx + 1;
            if animation.current_col_idx == animation.frames.len() as i32 {
                animation.current_col_idx = 0;
            }

            let anim_idx_mini = animation.current_col_idx * animation.row_cnt;
            let anim_idx_maxi = anim_idx_mini + animation.row_cnt - 1;

            let min_u = anim_idx_mini as usize;
            let max_u = anim_idx_maxi as usize;

            if sprite.index < min_u {
                sprite.index = min_u;
            } else if sprite.index >= max_u {
                sprite.index = max_u;
            } else {
                sprite.index = max_u;
            }
        }
    }
}

fn get_animation_from_movement(movement: &Movement) -> AnimationState {
    if movement.dir.y <= 0.0 {
        if movement.dir.x > 0.0 {
            return AnimationState::FrontRight;
        } else if movement.dir.x < 0.0 {
            return AnimationState::FrontLeft;
        }
    } else if movement.dir.y > 0.0 {
        if movement.dir.x > 0.0 {
            return AnimationState::BackRight;
        } else if movement.dir.x < 0.0 {
            return AnimationState::BackLeft;
        }
    }

    //NOTE: last idle anim
    if movement.last_moving_dir.y <= 0.0 {
        if movement.last_moving_dir.x > 0.0 {
            return AnimationState::IdleFrontRight;
        } else if movement.last_moving_dir.x < 0.0 {
            return AnimationState::IdleFrontLeft;
        }
    } else if movement.last_moving_dir.y > 0.0 {
        if movement.last_moving_dir.x > 0.0 {
            return AnimationState::IdleBackRight;
        } else if movement.last_moving_dir.x < 0.0 {
            return AnimationState::IdleBackLeft;
        }
    }

    AnimationState::IdleFrontLeft
}

fn entity_movement_animator_system(mut query: Query<(&mut Animation, &Movement)>) {
    for (mut animation, movement) in query.iter_mut() {
        animation.current_col_idx = get_animation_from_movement(movement) as i32;
    }
}

fn entity_movement_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut Movement)>) {
    let dt = time.delta().as_secs_f32() * 1000.0;
    for (mut transform, mut movement) in query.iter_mut() {
        if movement.dir.length() != 0.0 { movement.last_moving_dir = movement.dir; }
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

use std::error::Error;
fn read_anim_from_file<P: AsRef<std::path::Path>>(path: P) -> Animation {
    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);
    let u = Animation::deserialize(&mut de).unwrap();
    u
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/warior.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 8);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true))
        .insert(Movement{ dir: Vec2::default(), velocity: Vec2::default(), last_moving_dir: Vec2::default()})
        .insert(read_anim_from_file("assets/player.json"))
        .insert(create_player());
}