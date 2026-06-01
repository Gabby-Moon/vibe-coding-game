use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use rand::prelude::*;

const PLAYER_ACCELERATION: f32 = 900.0;
const PLAYER_MAX_SPEED: f32 = 350.0;
const PLAYER_FRICTION: f32 = 0.08;
const PLAYER_SIZE: f32 = 25.0;
const WALL_THICKNESS: f32 = 15.0;
const PATH_WIDTH: f32 = PLAYER_SIZE * 4.4;
const PICKUP_SIZE: f32 = 10.0;
const PICKUP_SPACING: f32 = 5.0;
const PICKUP_COUNT: usize = 80;

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Collectible {
    value: u32,
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct CountdownText;

#[derive(Resource)]
struct Score(u32);

#[derive(Resource)]
struct GameTimer {
    countdown: f32,
    remaining: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Score(0))
        .insert_resource(GameTimer {
            countdown: 3.0,
            remaining: 60.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_timers_text,
            player_movement.after(update_timers_text),
            resolve_wall_collisions.after(player_movement),
            collect_pickups.after(player_movement),
            update_score_text.after(collect_pickups),
        ))
        .run();
}

fn setup(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2dBundle::default());
    let window = window_query.get_single().unwrap();
    let wall_rects = spawn_maze(&mut commands, window);
    spawn_pickups(&mut commands, window, &wall_rects);
    spawn_score_ui(&mut commands);
    spawn_countdown_ui(&mut commands);

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

fn spawn_maze(commands: &mut Commands, window: &Window) -> Vec<(Vec2, Vec2)> {
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    let wall_color = Color::rgb(0.2, 0.2, 0.2);

    let walls = vec![
        // border walls
        (Vec2::new(0.0, half_height - WALL_THICKNESS / 2.0), Vec2::new(window.width(), WALL_THICKNESS)),
        (Vec2::new(0.0, -half_height + WALL_THICKNESS / 2.0), Vec2::new(window.width(), WALL_THICKNESS)),
        (Vec2::new(-half_width + WALL_THICKNESS / 2.0, 0.0), Vec2::new(WALL_THICKNESS, window.height())),
        (Vec2::new(half_width - WALL_THICKNESS / 2.0, 0.0), Vec2::new(WALL_THICKNESS, window.height())),

        // interior grouped blocks
        (Vec2::new(-PATH_WIDTH, PATH_WIDTH / 2.0), Vec2::new(WALL_THICKNESS, PATH_WIDTH * 1.2)),
        (Vec2::new(PATH_WIDTH, -PATH_WIDTH / 2.0), Vec2::new(WALL_THICKNESS, PATH_WIDTH * 1.2)),
        (Vec2::new(-PATH_WIDTH / 2.0, PATH_WIDTH), Vec2::new(PATH_WIDTH * 1.2, WALL_THICKNESS)),
        (Vec2::new(PATH_WIDTH / 2.0, -PATH_WIDTH), Vec2::new(PATH_WIDTH * 1.2, WALL_THICKNESS)),

        (Vec2::new(-PATH_WIDTH * 1.5, 0.0), Vec2::new(WALL_THICKNESS, PATH_WIDTH * 0.8)),
        (Vec2::new(PATH_WIDTH * 1.5, 0.0), Vec2::new(WALL_THICKNESS, PATH_WIDTH * 0.8)),
        (Vec2::new(0.0, -PATH_WIDTH * 1.5), Vec2::new(PATH_WIDTH * 0.8, WALL_THICKNESS)),
        (Vec2::new(0.0, PATH_WIDTH * 1.5), Vec2::new(PATH_WIDTH * 0.8, WALL_THICKNESS)),

        // extra obstacles around the edge
        (Vec2::new(-half_width / 3.0, half_height / 3.0), Vec2::new(WALL_THICKNESS, PATH_WIDTH * 0.7)),
        (Vec2::new(half_width / 3.0, -half_height / 3.0), Vec2::new(WALL_THICKNESS, PATH_WIDTH * 0.7)),
        (Vec2::new(half_width / 3.0, half_height / 3.0), Vec2::new(PATH_WIDTH * 0.7, WALL_THICKNESS)),
        (Vec2::new(-half_width / 3.0, -half_height / 3.0), Vec2::new(PATH_WIDTH * 0.7, WALL_THICKNESS)),
    ];

    for (position, size) in &walls {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: wall_color,
                    custom_size: Some(*size),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                ..default()
            })
            .insert(Wall);
    }

    walls
}

fn spawn_score_ui(commands: &mut Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.14, 0.14, 0.14, 0.5)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle {
                            text: Text::from_section(
                                "Time: 60",
                                TextStyle {
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            )
                            .with_alignment(TextAlignment::Center),
                            style: Style {
                                margin: UiRect::right(Val::Px(20.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(TimerText);

                    parent
                        .spawn(TextBundle {
                            text: Text::from_section(
                                "Score: 0",
                                TextStyle {
                                    font_size: 34.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            )
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        })
                        .insert(ScoreText);
                });
        });
}

fn spawn_countdown_ui(commands: &mut Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Auto,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                top: Val::Percent(45.0),
                left: Val::Px(0.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.75)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        "3",
                        TextStyle {
                            font_size: 80.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    ..default()
                })
                .insert(CountdownText);
        });
}

fn spawn_pickups(commands: &mut Commands, window: &Window, wall_rects: &[(Vec2, Vec2)]) {
    let mut rng = thread_rng();
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    let placement_margin = WALL_THICKNESS / 2.0 + PICKUP_SIZE / 2.0 + PICKUP_SPACING;
    let min_x = -half_width + placement_margin;
    let max_x = half_width - placement_margin;
    let min_y = -half_height + placement_margin;
    let max_y = half_height - placement_margin;

    let mut placed_positions = Vec::new();
    let max_attempts = 1000;

    for _ in 0..PICKUP_COUNT {
        let mut attempts = 0;
        while attempts < max_attempts {
            attempts += 1;
            let candidate = Vec2::new(rng.gen_range(min_x..max_x), rng.gen_range(min_y..max_y));
            if is_valid_pickup_position(candidate, wall_rects, &placed_positions) {
                let (color, value) = match rng.gen_range(0..6) {
                    0 => (Color::RED, 50),
                    1 => (Color::YELLOW, 100),
                    2 => (Color::GREEN, 20),
                    3 => (Color::BLUE, 5),
                    4 => (Color::PURPLE, 5),
                    _ => (Color::ORANGE, 5),
                };

                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Some(Vec2::splat(PICKUP_SIZE)),
                            ..default()
                        },
                        transform: Transform::from_xyz(candidate.x, candidate.y, 0.0),
                        ..default()
                    })
                    .insert(Collectible { value });

                placed_positions.push(candidate);
                break;
            }
        }
    }
}

fn is_valid_pickup_position(candidate: Vec2, wall_rects: &[(Vec2, Vec2)], existing: &[Vec2]) -> bool {
    let pickup_half = Vec2::splat(PICKUP_SIZE / 2.0 + PICKUP_SPACING);

    for (wall_pos, wall_size) in wall_rects {
        let wall_half = *wall_size / 2.0;
        let delta = candidate - *wall_pos;
        let overlap_x = pickup_half.x + wall_half.x - delta.x.abs();
        let overlap_y = pickup_half.y + wall_half.y - delta.y.abs();
        if overlap_x > 0.0 && overlap_y > 0.0 {
            return false;
        }
    }

    let min_center_distance = PICKUP_SIZE + PICKUP_SPACING;
    for other in existing {
        if candidate.distance_squared(*other) < min_center_distance * min_center_distance {
            return false;
        }
    }

    true
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    timer: Res<GameTimer>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    if timer.countdown > 0.0 || timer.remaining <= 0.0 {
        return;
    }

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

fn resolve_wall_collisions(
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    wall_query: Query<(&Transform, &Sprite), (With<Wall>, Without<Player>)>,
) {
    if let Ok((mut player_transform, mut player_velocity)) = player_query.get_single_mut() {
        let player_pos = player_transform.translation.truncate();
        let player_half = Vec2::splat(PLAYER_SIZE / 2.0);

        for (wall_transform, wall_sprite) in wall_query.iter() {
            if let Some(wall_size) = wall_sprite.custom_size {
                let wall_pos = wall_transform.translation.truncate();
                let wall_half = wall_size / 2.0;

                let delta = player_pos - wall_pos;
                let overlap_x = player_half.x + wall_half.x - delta.x.abs();
                let overlap_y = player_half.y + wall_half.y - delta.y.abs();

                if overlap_x > 0.0 && overlap_y > 0.0 {
                    if overlap_x < overlap_y {
                        if delta.x > 0.0 {
                            player_transform.translation.x += overlap_x;
                        } else {
                            player_transform.translation.x -= overlap_x;
                        }
                        player_velocity.0.x = 0.0;
                    } else {
                        if delta.y > 0.0 {
                            player_transform.translation.y += overlap_y;
                        } else {
                            player_transform.translation.y -= overlap_y;
                        }
                        player_velocity.0.y = 0.0;
                    }
                }
            }
        }
    }
}

fn collect_pickups(
    mut commands: Commands,
    mut score: ResMut<Score>,
    player_query: Query<&Transform, With<Player>>,
    pickup_query: Query<(Entity, &Transform, &Collectible)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation.truncate();
        let player_half = Vec2::splat(PLAYER_SIZE / 2.0);
        let pickup_half = Vec2::splat(PICKUP_SIZE / 2.0);

        for (entity, pickup_transform, collectible) in pickup_query.iter() {
            let pickup_pos = pickup_transform.translation.truncate();
            let delta = player_pos - pickup_pos;
            let overlap_x = player_half.x + pickup_half.x - delta.x.abs();
            let overlap_y = player_half.y + pickup_half.y - delta.y.abs();

            if overlap_x > 0.0 && overlap_y > 0.0 {
                score.0 += collectible.value;
                commands.entity(entity).despawn();
                info!("Score: {}", score.0);
            }
        }
    }
}

fn update_score_text(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = format!("Score: {}", score.0);
    }
}

fn update_timers_text(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    score: Res<Score>,
    mut queries: ParamSet<(
        Query<&mut Text, With<CountdownText>>,
        Query<&mut Text, With<TimerText>>,
    )>,
) {
    if timer.countdown > 0.0 {
        timer.countdown -= time.delta_seconds();
        if timer.countdown < 0.0 {
            timer.countdown = 0.0;
        }
    } else if timer.remaining > 0.0 {
        timer.remaining -= time.delta_seconds();
        if timer.remaining < 0.0 {
            timer.remaining = 0.0;
        }
    }

    if let Ok(mut text) = queries.p0().get_single_mut() {
        if timer.countdown > 0.0 {
            text.sections[0].value = timer.countdown.ceil().to_string();
        } else if timer.remaining > 0.0 && timer.remaining <= 10.0 {
            text.sections[0].value = timer.remaining.ceil().to_string();
        } else if timer.remaining <= 0.0 {
            text.sections[0].value = format!("Good Job! You got {} points!", score.0);
        } else {
            text.sections[0].value = String::new();
        }
    }

    if let Ok(mut text) = queries.p1().get_single_mut() {
        text.sections[0].value = format!("Time: {}", timer.remaining.ceil() as u32);
    }
}
