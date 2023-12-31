use std::{
    collections::HashSet,
    iter::{repeat, Flatten, Repeat},
};

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle,
};

mod constants {
    use bevy::prelude::*;
    // These constants are defined in `Transform` units.
    // Using the default 2D camera they correspond 1:1 with screen pixels.
    pub const PADDLE_SIZE: Vec3 = Vec3::new(20.0, 120.0, 0.0);
    pub const GAP_BETWEEN_PADDLE_AND_WALL: f32 = 60.0;
    pub const PADDLE_SPEED: f32 = 500.0;
    // How close can the paddle get to the wall
    pub const PADDLE_PADDING: f32 = 10.0;

    // We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
    pub const BALL_STARTING_POSITION: Vec3 = Vec3::new(-50.0, 0.0, 1.0);
    pub const BALL_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
    pub const BALL_SPEED: f32 = 400.0;
    pub const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

    pub const WALL_THICKNESS: f32 = 10.0;
    // x coordinates
    pub const LEFT_WALL: f32 = -450.;
    pub const RIGHT_WALL: f32 = 450.;
    // y coordinates
    pub const BOTTOM_WALL: f32 = -300.;
    pub const TOP_WALL: f32 = 300.;

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    pub const PADDLE_TOP_BOUND: f32 =
        TOP_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.y / 2.0 - PADDLE_PADDING;
    pub const PADDLE_BOTTOM_BOUND: f32 =
        BOTTOM_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.y / 2.0 + PADDLE_PADDING;

    // These values are exact
    pub const BACKGROUND_COLOR: Color = Color::BLACK;
    pub const PADDLE_COLOR: Color = Color::WHITE;
    pub const BALL_COLOR: Color = Color::RED;
    pub const WALL_COLOR: Color = Color::DARK_GRAY;

    pub const MAX_AI_PADDLE_SPEED: f32 = 500.0;

    pub const SCOREBOARD_FONT_SIZE: f32 = 40.0;
    pub const SCOREBOARD_PADDING_X: f32 =
        WALL_THICKNESS + GAP_BETWEEN_PADDLE_AND_WALL + (RIGHT_WALL - LEFT_WALL) / 5.0;
    pub const SCOREBOARD_PADDING_Y: f32 = (TOP_WALL - BOTTOM_WALL) / 10.0 + WALL_THICKNESS;
}

mod entities {
    use super::constants::*;
    use bevy::prelude::*;

    #[derive(Component, Debug, Clone, Hash, PartialEq, Eq)]
    pub struct Paddle;

    #[derive(Component, Debug, Hash, PartialEq, Eq)]
    pub struct Player;

    #[derive(Component, Debug, Hash, PartialEq, Eq)]
    pub struct AI;

    #[derive(Component, Debug, Clone, Hash, PartialEq, Eq)]
    pub struct Ball;

    #[derive(Component, Debug)]
    pub struct Collider;

    #[derive(Component, Deref, DerefMut)]
    pub struct Velocity(pub Vec2);

    #[derive(Component, Debug)]
    pub struct ScoreboardText;

    // This bundle is a collection of the components that define a "wall" in our game
    #[derive(Bundle)]
    pub struct Walls {
        pub sprite_bundle: SpriteBundle,
        pub collider: Collider,
        pub side: WallSide,
    }

    #[derive(Component, Debug, Clone, Hash, PartialEq, Eq)]
    pub enum WallSide {
        Top,
        Bottom,
        Player,
        Enemy,
    }

    impl WallSide {
        pub fn position(&self) -> Vec2 {
            match self {
                WallSide::Enemy => Vec2::new(LEFT_WALL, 0.),
                WallSide::Player => Vec2::new(RIGHT_WALL, 0.),
                WallSide::Bottom => Vec2::new(0., BOTTOM_WALL),
                WallSide::Top => Vec2::new(0., TOP_WALL),
            }
        }

        pub fn size(&self) -> Vec2 {
            let arena_height = TOP_WALL - BOTTOM_WALL;
            let arena_width = RIGHT_WALL - LEFT_WALL;
            // Make sure we haven't messed up our constants
            assert!(arena_height > 0.0);
            assert!(arena_width > 0.0);

            match self {
                WallSide::Enemy | WallSide::Player => {
                    Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
                }
                WallSide::Bottom | WallSide::Top => {
                    Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
                }
            }
        }
    }

    impl Walls {
        pub fn new(location: WallSide) -> Self {
            Self {
                sprite_bundle: SpriteBundle {
                    transform: Transform {
                        translation: location.position().extend(0.0),
                        scale: location.size().extend(1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: WALL_COLOR,
                        ..default()
                    },
                    ..default()
                },
                collider: Collider,
                side: location,
            }
        }
    }
}

fn spawn_ball(
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> (
    MaterialMesh2dBundle<ColorMaterial>,
    entities::Ball,
    entities::Velocity,
) {
    (
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(constants::BALL_COLOR)),
            transform: Transform::from_translation(constants::BALL_STARTING_POSITION)
                .with_scale(constants::BALL_SIZE),
            ..default()
        },
        entities::Ball,
        entities::Velocity(constants::INITIAL_BALL_DIRECTION.normalize() * constants::BALL_SPEED),
    )
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum Owner {
    Player,
    AI,
}

#[derive(Debug, Event, Hash, PartialEq, Eq)]
enum CollisionEvent {
    Wall(entities::Ball, entities::WallSide),
    Paddle(entities::Ball, entities::Paddle, Owner),
}

#[derive(Resource, Default)]
pub struct Scoreboard {
    pub ai: usize,
    pub player: usize,
}

// provides an alternating collision sound.
#[derive(Resource)]
struct CollisionSound {
    iter: Flatten<Repeat<Vec<Handle<AudioSource>>>>,
    // the same collion can occur in contiguous frames, debounce them with this instant
    last: f32,
}

impl FromIterator<Handle<AudioSource>> for CollisionSound {
    fn from_iter<T: IntoIterator<Item = Handle<AudioSource>>>(iter: T) -> Self {
        CollisionSound {
            iter: repeat(iter.into_iter().collect::<Vec<_>>()).flatten(),
            last: 0.,
        }
    }
}

impl CollisionSound {
    // returns a sound if we haven't played one recently, otherwise None
    fn next(&mut self, time: f32) -> Option<Handle<AudioSource>> {
        if time - self.last < 0.05 {
            return None;
        }

        self.last = time;
        self.iter.next()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    let player_paddle_x = constants::RIGHT_WALL - constants::GAP_BETWEEN_PADDLE_AND_WALL;
    let ai_paddle_x = constants::LEFT_WALL + constants::GAP_BETWEEN_PADDLE_AND_WALL;

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(player_paddle_x, 0.0, 0.0),
                scale: constants::PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: constants::PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        entities::Player,
        entities::Paddle,
        entities::Collider,
    ));

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(ai_paddle_x, 0.0, 0.0),
                scale: constants::PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: constants::PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        entities::AI,
        entities::Paddle,
        entities::Collider,
    ));

    // Walls
    commands.spawn(entities::Walls::new(entities::WallSide::Top));
    commands.spawn(entities::Walls::new(entities::WallSide::Bottom));
    commands.spawn(entities::Walls::new(entities::WallSide::Enemy));
    commands.spawn(entities::Walls::new(entities::WallSide::Player));

    // Ball
    commands.spawn(spawn_ball(&mut materials, &mut meshes));

    // AI Score
    commands.spawn((
        Text2dBundle {
            text: Text::from_sections([TextSection::from_style(TextStyle {
                font_size: constants::SCOREBOARD_FONT_SIZE,
                color: Color::GRAY,
                ..default()
            })]),
            transform: Transform::from_translation(Vec3::new(
                constants::LEFT_WALL + constants::SCOREBOARD_PADDING_X,
                constants::TOP_WALL - constants::SCOREBOARD_PADDING_Y,
                1.0,
            )),
            ..default()
        },
        entities::ScoreboardText,
        entities::AI,
    ));

    // Player Score
    commands.spawn((
        Text2dBundle {
            text: Text::from_sections([TextSection::from_style(TextStyle {
                font_size: constants::SCOREBOARD_FONT_SIZE,
                color: Color::GRAY,
                ..default()
            })]),
            transform: Transform::from_translation(Vec3::new(
                constants::RIGHT_WALL - constants::SCOREBOARD_PADDING_X,
                constants::TOP_WALL - constants::SCOREBOARD_PADDING_Y,
                1.0,
            )),
            ..default()
        },
        entities::ScoreboardText,
        entities::Player,
    ));

    commands.insert_resource(CollisionSound::from_iter([
        asset_server.load("high_beep_short.ogg"),
        asset_server.load("low_beep_short.ogg"),
    ]));
}

fn move_player_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, (With<entities::Player>, With<entities::Paddle>)>,
    time: Res<Time>,
) {
    let mut paddle_transform = query.single_mut();
    let direction = if keyboard_input.any_pressed([KeyCode::Up, KeyCode::W, KeyCode::K]) {
        1.0
    } else if keyboard_input.any_pressed([KeyCode::Down, KeyCode::S, KeyCode::J]) {
        -1.0
    } else {
        0.0
    };

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position =
        paddle_transform.translation.y + direction * constants::PADDLE_SPEED * time.delta_seconds();

    paddle_transform.translation.y =
        new_paddle_position.clamp(constants::PADDLE_BOTTOM_BOUND, constants::PADDLE_TOP_BOUND);
}

fn enemy_paddle_ai(
    mut paddle_query: Query<&mut Transform, (With<entities::AI>, With<entities::Paddle>)>,
    ball_query: Query<&Transform, (With<entities::Ball>, Without<entities::AI>)>,
    time: Res<Time>,
) {
    let mut paddle_transform = paddle_query.single_mut();

    let ball_transform = ball_query.single();

    // anticipate next ball position, adjust paddle. clamp to a player speed
    let next_y = ball_transform.translation.y;

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position = paddle_transform.translation.y
        + (next_y - paddle_transform.translation.y).clamp(
            -constants::MAX_AI_PADDLE_SPEED * time.delta_seconds(),
            constants::MAX_AI_PADDLE_SPEED * time.delta_seconds(),
        );

    paddle_transform.translation.y =
        new_paddle_position.clamp(constants::PADDLE_BOTTOM_BOUND, constants::PADDLE_TOP_BOUND);
}

fn apply_velocity(mut query: Query<(&mut Transform, &entities::Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn generate_ball_collide_events(
    ball_q: Query<(&entities::Ball, &Transform), With<entities::Ball>>,
    collider_q: Query<
        (
            &Transform,
            (Option<&entities::AI>, Option<&entities::Player>),
            (Option<&entities::WallSide>, Option<&entities::Paddle>),
        ),
        With<entities::Collider>,
    >,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (ball, ball_transform) = ball_q.single();
    let ball_size = ball_transform.scale.truncate();
    let mut events = HashSet::new();

    // check collision with walls
    for (transform, player_kind, entity_kind) in &collider_q {
        if collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        )
        .is_none()
        {
            continue;
        }

        let ball = ball.to_owned();
        // yuck
        let ev = match (player_kind, entity_kind) {
            (_, (Some(ws), None)) => CollisionEvent::Wall(ball, ws.clone()),
            ((Some(_), None), (None, Some(pd))) => {
                CollisionEvent::Paddle(ball, pd.clone(), Owner::AI)
            }
            ((None, Some(_)), (None, Some(pd))) => {
                CollisionEvent::Paddle(ball, pd.clone(), Owner::Player)
            }
            other => unreachable!("cannot reach {other:?}"),
        };
        println!("Collision: {ev:?}");
        events.insert(ev);
    }

    for ev in events {
        collision_events.send(ev);
    }
}

fn check_ball_bounce_collisions(
    mut ball_query: Query<(&mut entities::Velocity, &Transform), With<entities::Ball>>,
    collider_query: Query<&Transform, With<entities::Collider>>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();

    for transform in &collider_query {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            // reflect the ball when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // only reflect if the ball's velocity is going in the opposite direction of the
            // collision
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => (),
            }

            // reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            // reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn tally_score(mut collision_events: EventReader<CollisionEvent>, mut scores: ResMut<Scoreboard>) {
    for ev in collision_events.read() {
        match ev {
            CollisionEvent::Paddle(_, _, _) => (),
            CollisionEvent::Wall(_, entities::WallSide::Enemy) => scores.ai += 1,
            CollisionEvent::Wall(_, entities::WallSide::Player) => scores.player += 1,
            CollisionEvent::Wall(_, _) => (),
        }
    }
}

fn update_scoreboard(
    mut player_scoreboard: Query<
        &mut Text,
        (
            With<entities::ScoreboardText>,
            With<entities::Player>,
            Without<entities::AI>,
        ),
    >,
    mut ai_scoreboard: Query<
        &mut Text,
        (
            With<entities::ScoreboardText>,
            With<entities::AI>,
            Without<entities::Player>,
        ),
    >,
    scores: Res<Scoreboard>,
) {
    player_scoreboard.single_mut().sections[0].value = scores.player.to_string();
    ai_scoreboard.single_mut().sections[0].value = scores.ai.to_string();
}

fn handle_round_over(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ball_query: Query<Entity, With<entities::Ball>>,
) {
    use entities::WallSide::*;

    let Some(_) = collision_events
        .read()
        .find(|ev| matches!(ev, CollisionEvent::Wall(_, Player | Enemy)))
    else {
        return;
    };

    let ball = ball_query.single();
    commands.entity(ball).despawn();
    commands.spawn(spawn_ball(&mut materials, &mut meshes));
}

fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut sound: ResMut<CollisionSound>,
    time: Res<Time<Real>>,
) {
    if collision_events.read().any(|ev| {
        !matches!(
            ev,
            CollisionEvent::Wall(_, entities::WallSide::Player | entities::WallSide::Enemy)
        ) || matches!(ev, CollisionEvent::Paddle(_, _, _))
    }) {
        collision_events.clear(); // consume them all

        let time = time.elapsed_seconds();

        if let Some(source) = sound.next(time) {
            commands.spawn(AudioBundle {
                source,
                settings: PlaybackSettings::DESPAWN,
            });
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(constants::BACKGROUND_COLOR))
        .insert_resource(Scoreboard::default())
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        .add_systems(
            FixedUpdate,
            (
                move_player_paddle,
                generate_ball_collide_events,
                // move the ball after making events or we'll miss events
                apply_velocity,
                check_ball_bounce_collisions,
                tally_score,
                update_scoreboard,
                enemy_paddle_ai,
                handle_round_over,
                play_collision_sound,
            ),
        )
        // .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}
