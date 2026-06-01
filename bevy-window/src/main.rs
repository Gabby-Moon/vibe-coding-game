use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};

const PLAYER_ACCELERATION: f32 = 900.0;
const PLAYER_MAX_SPEED: f32 = 350.0;
const PLAYER_FRICTION: f32 = 0.08;
const PLAYER_SIZE: f32 = 25.0;

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec2);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, player_movement)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.7, 0.9),
                custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Player)
        .insert(Velocity::default());
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    if let Ok((mut transform, mut velocity)) = query.get_single_mut() {
        let window = window_query.get_single().unwrap();
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
            direction.x += 1.0;
        }

        if direction != Vec2::ZERO {
            let direction = direction.normalize();
            velocity.0 += direction * PLAYER_ACCELERATION * time.delta_seconds();
        }

        let speed = velocity.0.length();
        if speed > PLAYER_MAX_SPEED {
            velocity.0 = velocity.0.normalize() * PLAYER_MAX_SPEED;
        }

        if direction == Vec2::ZERO {
            let friction = 1.0 - PLAYER_FRICTION * time.delta_seconds();
            velocity.0 *= friction.clamp(0.0, 1.0);
            if velocity.0.length_squared() < 1.0 {
                velocity.0 = Vec2::ZERO;
            }
        }

        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();

        let half_size = PLAYER_SIZE / 2.0;
        let x_bound = window.width() / 2.0 - half_size;
        let y_bound = window.height() / 2.0 - half_size;

        transform.translation.x = transform.translation.x.clamp(-x_bound, x_bound);
        transform.translation.y = transform.translation.y.clamp(-y_bound, y_bound);
    }
}
