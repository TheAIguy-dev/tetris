use std::process::exit;

use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
};
use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::close_on_esc;
use iyes_perf_ui::prelude::*;
use rand::random;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 24;

#[derive(Clone, Component, Copy, Eq, PartialEq)]
enum Shape {
    Z,
    S,
    I,
    T,
    O,
    L,
    J,
}
impl Shape {
    fn random() -> Self {
        match random::<u8>() % 7 {
            0 => Shape::Z,
            1 => Shape::S,
            2 => Shape::I,
            3 => Shape::T,
            4 => Shape::O,
            5 => Shape::L,
            6 => Shape::J,
            _ => unreachable!(),
        }
    }

    fn blocks(&self) -> [[i32; 2]; 4] {
        use Shape::*;
        match self {
            Z => [[0, -1], [0, 0], [-1, 0], [-1, 1]],
            S => [[0, -1], [0, 0], [1, 0], [1, 1]],
            I => [[0, -1], [0, 0], [0, 1], [0, 2]],
            T => [[-1, 0], [0, 0], [1, 0], [0, 1]],
            O => [[0, 0], [1, 0], [0, 1], [1, 1]],
            L => [[-1, -1], [0, -1], [0, 0], [0, 1]],
            J => [[1, -1], [0, -1], [0, 0], [0, 1]],
        }
    }

    fn color(&self) -> Color {
        use Shape::*;
        match self {
            Z => Color::RED,
            S => Color::GREEN,
            I => Color::BLUE,
            T => Color::YELLOW,
            O => Color::ORANGE,
            L => Color::PURPLE,
            J => Color::PINK,
        }
    }
}

#[derive(Component)]
struct Piece {
    shape: Shape,
    blocks: [Entity; 4],
    blocks_pos: [[i32; 2]; 4],
    pos: [i32; 2],
    prev_pos: [i32; 2],
    rot: u8,
    prev_rot: u8,
}
impl Piece {
    fn new() -> Self {
        let shape: Shape = Shape::random();
        Self {
            shape,
            blocks: [Entity::PLACEHOLDER; 4],
            blocks_pos: shape.blocks(),
            pos: [BOARD_WIDTH as i32 / 2, 0],
            prev_pos: [BOARD_WIDTH as i32 / 2, 0],
            rot: 0,
            prev_rot: 0,
        }
    }

    fn get_block_transform(&self, i: usize) -> Transform {
        Transform::from_xyz(
            -(BOARD_WIDTH as f32 / 2.0) + self.blocks_pos[i][0] as f32 + self.pos[0] as f32,
            BOARD_HEIGHT as f32 / 2.0 - self.blocks_pos[i][1] as f32 - self.pos[1] as f32,
            0.0,
        )
    }

    fn rotate_right(&mut self) {
        if self.shape == Shape::O {
            return;
        }

        let mut new_blocks_pos: [[i32; 2]; 4] = [[0; 2]; 4];
        for i in 0..4 {
            new_blocks_pos[i] = [-self.blocks_pos[i][1], self.blocks_pos[i][0]];
        }

        self.blocks_pos = new_blocks_pos;
        self.rot = (self.rot + 3) % 4;
    }

    fn rotate_left(&mut self) {
        if self.shape == Shape::O {
            return;
        }

        let mut new_blocks_pos: [[i32; 2]; 4] = [[0; 2]; 4];
        for i in 0..4 {
            new_blocks_pos[i] = [self.blocks_pos[i][1], -self.blocks_pos[i][0]];
        }

        self.blocks_pos = new_blocks_pos;
        self.rot = (self.rot + 1) % 4;
    }
}

#[derive(Component)]
struct DeadPiece {
    pos: [i32; 2],
}
impl DeadPiece {
    fn get_block_transform(&self) -> Transform {
        Transform::from_xyz(
            -(BOARD_WIDTH as f32 / 2.0) + self.pos[0] as f32,
            BOARD_HEIGHT as f32 / 2.0 - self.pos[1] as f32,
            0.0,
        )
    }
}

#[derive(Resource)]
struct MoveDownTimer(Timer);

#[derive(Resource)]
struct Board([[Option<Entity>; BOARD_WIDTH]; BOARD_HEIGHT]);
impl Board {
    fn get_block(&self, x: i32, y: i32) -> Option<Entity> {
        self.0[y as usize][x as usize]
    }

    fn set_block(&mut self, x: i32, y: i32, entity: Option<Entity>) {
        self.0[y as usize][x as usize] = entity;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::ERROR,
            ..default()
        }))
        // diagnostics
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(EntityCountDiagnosticsPlugin)
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        // resources
        .insert_resource(MoveDownTimer(Timer::from_seconds(
            0.3,
            TimerMode::Repeating,
        )))
        .insert_resource(Board([[None; BOARD_WIDTH]; BOARD_HEIGHT]))
        // systems
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .add_systems(
            Update,
            (
                (
                    rotate_piece_right.run_if(input_just_pressed(KeyCode::KeyQ)),
                    rotate_piece_left.run_if(input_just_pressed(KeyCode::KeyR)),
                    move_piece_right.run_if(input_just_pressed(KeyCode::KeyA)),
                    move_piece_left.run_if(input_just_pressed(KeyCode::KeyD)),
                    move_piece_down,
                    check_collision,
                    update_piece,
                )
                    .chain()
                    .distributive_run_if(piece_present),
                (update_board, update_dead_pieces, spawn_piece)
                    .chain()
                    .run_if(not(piece_present)),
            ),
        )
        // run
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // commands.spawn(PerfUiCompleteBundle::default());

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, -40.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(
            Plane3d::default()
                .mesh()
                .size(BOARD_WIDTH as f32 + 5.0, 5.0),
        ),
        material: materials.add(Color::BLACK),
        transform: Transform::from_xyz(0.0, -(BOARD_HEIGHT as f32) / 2.0 + 0.5, 0.0),
        ..default()
    });
}

fn spawn_piece(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut piece: Piece = Piece::new();

    for i in 0..4 {
        let entity = commands
            .spawn(PbrBundle {
                mesh: meshes.add(Cuboid::from_size(Vec3::ONE)),
                material: materials.add(piece.shape.color()),
                transform: piece.get_block_transform(i),
                ..default()
            })
            .id();
        piece.blocks[i] = entity;
    }

    commands.spawn(piece);
}

fn rotate_piece_right(mut query: Query<&mut Piece>) {
    let mut piece: Mut<Piece> = query.single_mut();

    piece.rotate_right();
}

fn rotate_piece_left(mut query: Query<&mut Piece>) {
    let mut piece: Mut<Piece> = query.single_mut();

    piece.rotate_left();
}

fn move_piece_right(mut query: Query<&mut Piece>) {
    let mut piece: Mut<Piece> = query.single_mut();
    piece.pos[0] += 1;
}

fn move_piece_left(mut query: Query<&mut Piece>) {
    let mut piece: Mut<Piece> = query.single_mut();
    piece.pos[0] -= 1;
}

fn move_piece_down(
    time: Res<Time>,
    mut timer: ResMut<MoveDownTimer>,
    mut piece: Query<&mut Piece>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut piece: Mut<Piece> = piece.single_mut();
        piece.pos[1] += 1;
    }
}

fn check_collision(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Piece)>,
    mut board: ResMut<Board>,
) {
    let (entity, mut piece) = query.single_mut();
    for i in 0..4 {
        let x: i32 = piece.pos[0] + piece.blocks_pos[i][0];
        let y: i32 = piece.pos[1] + piece.blocks_pos[i][1];
        if x < 0
            || x >= BOARD_WIDTH as i32
            || y >= BOARD_HEIGHT as i32
            || y < 0
            || board.get_block(x, y).is_some()
        {
            // Collision!

            // Reset piece to previous state
            piece.pos = piece.prev_pos;
            while piece.rot != piece.prev_rot {
                piece.rotate_right();
            }

            // Despawn piece, insert blocks into board
            if y >= 0
                && (..BOARD_WIDTH).contains(&(x as usize))
                && (y >= BOARD_HEIGHT as i32 || board.get_block(x, y).is_some())
            {
                for (i, entity) in piece.blocks.into_iter().enumerate() {
                    let x = piece.pos[0] + piece.blocks_pos[i][0];
                    let y = piece.pos[1] + piece.blocks_pos[i][1];

                    // Game over
                    if y == -1 {
                        println!("Game over!");
                        exit(0);
                    }

                    commands.entity(entity).insert(DeadPiece { pos: [x, y] });
                    board.set_block(x, y, Some(entity));
                }
                commands.entity(entity).despawn();
            }

            return;
        }
    }

    // No collision, update previous state
    piece.prev_pos = piece.pos;
    piece.prev_rot = piece.rot;
}

fn update_board(
    mut commands: Commands,
    mut board: ResMut<Board>,
    mut query: Query<&mut DeadPiece>,
) {
    for i in (0..BOARD_HEIGHT).rev() {
        while board.0[i].iter().all(Option::is_some) {
            for entity in board.0[i] {
                commands.entity(entity.unwrap()).despawn();
            }
            board.0[i] = [None; BOARD_WIDTH];
            board.0[..=i].rotate_right(1);

            for entity in board.0[..=i].concat() {
                if let Some(entity) = entity {
                    if let Ok(mut dead_piece) = query.get_mut(entity) {
                        dead_piece.pos[1] += 1;
                    }
                }
            }
        }
    }
}

fn update_piece(mut commands: Commands, mut piece: Query<&mut Piece>) {
    let piece: Mut<Piece> = piece.single_mut();

    for i in 0..4 {
        commands
            .entity(piece.blocks[i])
            .insert(piece.get_block_transform(i));
    }
}

fn update_dead_pieces(mut commands: Commands, mut query: Query<(Entity, &mut DeadPiece)>) {
    for (entity, dead_piece) in query.iter_mut() {
        commands
            .entity(entity)
            .insert(dead_piece.get_block_transform());
    }
}

fn piece_present(piece: Query<&Piece>) -> bool {
    piece.iter().count() == 1
}
