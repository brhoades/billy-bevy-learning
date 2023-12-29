use bevy::prelude::*;

mod entities {
    use bevy::prelude::*;
    #[derive(Component)]
    pub(crate) struct Paddle;

    #[derive(Component)]
    pub(crate) struct Ball;
}

fn main() {
    App::new().run();
}
