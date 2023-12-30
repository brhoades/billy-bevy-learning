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
    pub const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
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

    // These values are exact
    pub const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
    pub const GAP_BETWEEN_BRICKS: f32 = 5.0;
    // These values are lower bounds, as the number of bricks is computed
    pub const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
    pub const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

    pub const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
    pub const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
    pub const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
    pub const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
}

mod entities {
    use super::constants::*;
    use bevy::prelude::*;
    #[derive(Component)]
    pub(crate) struct Paddle;

    #[derive(Component)]
    pub(crate) struct Ball;

    #[derive(Component)]
    pub(crate) struct Collider;

    // This bundle is a collection of the components that define a "wall" in our game
    #[derive(Bundle)]
    pub struct Walls {
        sprite_bundle: SpriteBundle,
        collider: Collider,
    }

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
        // This "builder method" allows us to reuse logic across our wall entities,
        // making our code easier to read and less prone to bugs when we change the logic
        pub fn new(location: WallSide) -> Self {
            Self {
                sprite_bundle: SpriteBundle {
                    transform: Transform {
                        // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                        // This is used to determine the order of our sprites
                        translation: location.position().extend(0.0),
                        // The z-scale of 2D objects must always be 1.0,
                        // or their ordering will be affected in surprising ways.
                        // See https://github.com/bevyengine/bevy/issues/4149
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
            }
        }
    }
}

#[derive(Event, Default)]
struct CollisionEvent;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    let player_paddle_x = constants::RIGHT_WALL - constants::GAP_BETWEEN_PADDLE_AND_WALL;
    let enemy_paddle_x = constants::LEFT_WALL + constants::GAP_BETWEEN_PADDLE_AND_WALL;

    for offset_x in [player_paddle_x, enemy_paddle_x] {
        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(offset_x, 0.0, 0.0),
                    scale: constants::PADDLE_SIZE,
                    ..default()
                },
                sprite: Sprite {
                    color: constants::PADDLE_COLOR,
                    ..default()
                },
                ..default()
            },
            entities::Paddle,
            entities::Collider,
        ));
    }

    // Walls
    commands.spawn(entities::Walls::new(entities::WallSide::Top));
    commands.spawn(entities::Walls::new(entities::WallSide::Bottom));
    commands.spawn(entities::Walls::new(entities::WallSide::Enemy));
    commands.spawn(entities::Walls::new(entities::WallSide::Player));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(constants::BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        /*
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_paddle,
                check_for_collisions,
                play_collision_sound,
            )
                // `chain`ing systems together runs them in order
                .chain(),
        )
        */
        // .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}
