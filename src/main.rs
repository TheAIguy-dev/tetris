use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    input::common_conditions::{input_just_pressed, input_pressed},
    math::I16Vec2,
    prelude::{light_consts::lux::OVERCAST_DAY, *},
};
use iyes_perf_ui::prelude::*;
use rand::seq::SliceRandom;

fn rotate<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    if v.is_empty() {
        return v;
    }

    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().rev().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 24;

#[derive(Clone, Component, Copy, Debug, Eq, PartialEq)]
enum PieceType {
    O,
    I,
    S,
    Z,
    L,
    J,
    T,
}
impl PieceType {
    fn random() -> Self {
        use PieceType::*;
        *[O, I, S, Z, L, J, T]
            .as_slice()
            .choose(&mut rand::thread_rng())
            .unwrap()
    }

    fn get_blocks(&self) -> Vec<Vec<bool>> {
        use PieceType::*;
        match self {
            O => vec![vec![true, true], vec![true, true]],
            I => vec![vec![true], vec![true], vec![true], vec![true]],
            S => vec![vec![false, true, true], vec![true, true, false]],
            Z => vec![vec![true, true, false], vec![false, true, true]],
            L => vec![vec![true, false], vec![true, false], vec![true, true]],
            J => vec![vec![false, true], vec![false, true], vec![true, true]],
            T => vec![vec![true, true, true], vec![false, true, false]],
        }
    }

    fn get_color(&self) -> Color {
        use PieceType::*;
        match self {
            O => Color::YELLOW,
            I => Color::BLUE,
            S => Color::RED,
            Z => Color::GREEN,
            L => Color::ORANGE,
            J => Color::PINK,
            T => Color::PURPLE,
        }
    }
}

// #[derive(Clone, Component, Copy)]
// enum PieceRotation {
//     Zero,
//     Ninety,
//     OneEighty,
//     TwoSeventy,
// }
// impl PieceRotation {
//     fn new() -> Self {
//         PieceRotation::Zero
//     }
//
//     fn rotate(&mut self) {
//         *self = match self {
//             PieceRotation::Zero => PieceRotation::Ninety,
//             PieceRotation::Ninety => PieceRotation::OneEighty,
//             PieceRotation::OneEighty => PieceRotation::TwoSeventy,
//             PieceRotation::TwoSeventy => PieceRotation::Zero,
//         }
//     }
// }

#[derive(Component)]
struct PieceBlocks(pub Vec<Vec<bool>>);

#[derive(Clone, Component, Copy)]
struct Position(pub I16Vec2);

#[derive(Clone, Component, Copy)]
struct Velocity(pub I16Vec2);

#[derive(Component)]
struct Piece {
    pub piece_type: PieceType,
    pub blocks: PieceBlocks,
    pub position: Position,
    pub velocity: Velocity,
}
impl Piece {
    fn new() -> Self {
        let piece_type: PieceType = PieceType::random();
        Piece {
            piece_type,
            blocks: PieceBlocks(piece_type.get_blocks()),
            position: Position(I16Vec2::new(BOARD_WIDTH as i16 / 2, 0)),
            velocity: Velocity(I16Vec2::Y),
        }
    }

    fn rotate(&mut self) {
        self.blocks.0 = rotate(self.blocks.0.clone());
    }
}

#[derive(Component)]
struct ActivePieceBlock {
    position: Position,
    velocity: Velocity,
}

#[derive(Resource)]
struct MovementTimer(pub Timer);

#[derive(Resource)]
struct GameBoard(pub Vec<Vec<Option<PieceType>>>);
impl GameBoard {
    pub fn new() -> Self {
        let mut board: Vec<Vec<Option<PieceType>>> = vec![];
        for _ in 0..BOARD_HEIGHT {
            let mut row: Vec<Option<PieceType>> = vec![];
            for _ in 0..BOARD_WIDTH {
                row.push(None)
            }
            board.push(row);
        }
        GameBoard(board)
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // diagnostics
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(EntityCountDiagnosticsPlugin)
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        // resources
        .insert_resource(GameBoard::new())
        .insert_resource(MovementTimer(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )))
        // systems
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (
                    (
                        zero_velocity,
                        sync_velocity,
                        move_piece_left.run_if(input_just_pressed(KeyCode::KeyA)),
                        move_piece_right.run_if(input_just_pressed(KeyCode::KeyD)),
                        sync_velocity,
                        movement_immediate,
                        reset_velocity,
                        sync_velocity,
                    )
                        .chain(),
                    rotate_piece.run_if(input_just_pressed(KeyCode::KeyR)),
                    movement,
                    (reset_velocity, sync_velocity).chain(),
                )
                    .chain()
                    .distributive_run_if(not(no_active_piece)),
                spawn_piece.run_if(no_active_piece),
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PerfUiCompleteBundle::default());

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -35.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // // circular base
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Circle::new(4.0)),
    //     material: materials.add(Color::WHITE),
    //     transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    //     ..default()
    // });
    // // cube
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
    //     material: materials.add(Color::rgb_u8(124, 144, 255)),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..default()
    // });
    // // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    // // camera
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });
}

fn spawn_piece(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let piece: Piece = Piece::new();

    for y in 0..piece.blocks.0.len() {
        for x in 0..piece.blocks.0[0].len() {
            if piece.blocks.0[y][x] {
                commands.spawn((
                    ActivePieceBlock {
                        position: Position(piece.position.0 + I16Vec2::new(x as i16, y as i16)),
                        velocity: Velocity(piece.velocity.0),
                    },
                    PbrBundle {
                        mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
                        material: materials.add(piece.piece_type.get_color()),
                        transform: Transform::from_translation(Vec3::new(
                            -(BOARD_WIDTH as f32 / 2.0) + piece.position.0.x as f32 + x as f32,
                            (BOARD_HEIGHT as f32 / 2.0) - (piece.position.0.y as f32 + y as f32),
                            0.0,
                        )),
                        ..default()
                    },
                ));
            }
        }
    }

    commands.spawn(piece);
}

fn movement(
    mut commands: Commands,
    mut timer: ResMut<MovementTimer>,
    time: Res<Time>,
    piece: Query<(Entity, &mut Piece)>,
    mut query: Query<(Entity, &mut ActivePieceBlock, &mut Transform)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut exit: bool = false;

        for (_, block, _) in &query {
            let (pos, vel) = (block.position, block.velocity);

            let new_pos: I16Vec2 = pos.0 + vel.0;
            if new_pos.y >= BOARD_HEIGHT as i16 {
                exit = true;
                break;
            }
            if new_pos.x < 0 || new_pos.x >= BOARD_WIDTH as i16 {
                return;
            }
        }

        if exit {
            for (entity, ..) in &mut query {
                commands
                    .get_entity(entity)
                    .unwrap()
                    .remove::<ActivePieceBlock>();
            }
            commands.get_entity(piece.single().0).unwrap().despawn();
            return;
        }

        movement_immediate(piece, query);
    }
}

fn movement_immediate(
    mut piece: Query<(Entity, &mut Piece)>,
    mut query: Query<(Entity, &mut ActivePieceBlock, &mut Transform)>,
) {
    let vel: I16Vec2 = piece.single().1.velocity.0;
    dbg!(vel);
    piece.single_mut().1.position.0 += vel;
    for (_, mut block, mut transform) in query.iter_mut() {
        let vel: I16Vec2 = block.velocity.0;
        block.position.0 += vel;
        transform.translation = Vec3::new(
            -(BOARD_WIDTH as f32 / 2.0) + block.position.0.x as f32,
            (BOARD_HEIGHT as f32 / 2.0) - (block.position.0.y as f32),
            0.0,
        );
    }
}

fn rotate_piece(
    mut piece: Query<&mut Piece>,
    mut query: Query<(&mut ActivePieceBlock, &mut Transform)>,
) {
    let mut piece: Mut<Piece> = piece.single_mut();
    piece.rotate();

    let mut query = query.iter_mut();
    for y in 0..piece.blocks.0.len() {
        for x in 0..piece.blocks.0[0].len() {
            if piece.blocks.0[y][x] {
                let (mut block, mut transform) = query.next().unwrap();
                block.position.0 = piece.position.0 + I16Vec2::new(x as i16, y as i16);
                transform.translation = Vec3::new(
                    -(BOARD_WIDTH as f32 / 2.0) + block.position.0.x as f32,
                    (BOARD_HEIGHT as f32 / 2.0) - (block.position.0.y as f32),
                    0.0,
                );
            }
        }
    }
}

fn move_piece_left(mut piece: Query<&mut Piece>) {
    let mut piece: Mut<Piece> = piece.single_mut();
    piece.velocity.0 = I16Vec2::X;
}

fn move_piece_right(mut piece: Query<&mut Piece>) {
    let mut piece: Mut<Piece> = piece.single_mut();
    piece.velocity.0 = I16Vec2::NEG_X;
}

fn sync_velocity(piece: Query<&Piece>, mut query: Query<&mut ActivePieceBlock>) {
    let piece: &Piece = piece.single();

    for mut block in query.iter_mut() {
        block.velocity.0 = piece.velocity.0;
    }
}

fn reset_velocity(mut piece: Query<&mut Piece>) {
    let mut piece: Mut<Piece> = piece.single_mut();
    piece.velocity.0 = I16Vec2::Y;
}

fn zero_velocity(mut piece: Query<&mut Piece>) {
    let mut piece: Mut<Piece> = piece.single_mut();
    piece.velocity.0 = I16Vec2::ZERO;
}

fn no_active_piece(query: Query<&ActivePieceBlock>) -> bool {
    query.is_empty()
}
