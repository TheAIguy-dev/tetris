use std::f32::consts::PI;

use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
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

    fn size(&self) -> (usize, usize) {
        use PieceType::*;
        match self {
            O => (2, 2),
            I => (1, 3),
            S | Z | T => (3, 2),
            L | J => (2, 3),
        }
    }

    fn get_blocks(&self) -> Vec<Vec<bool>> {
        use PieceType::*;
        match self {
            O => vec![vec![true, true], vec![true, true]],
            I => vec![vec![true], vec![true], vec![true]],
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

#[derive(Bundle)]
struct Piece {
    pub piece_type: PieceType,
    pub blocks: PieceBlocks,
    pub position: Position,
    pub velocity: Velocity,
    // pub transform: Transform,
    // pub global_transform: GlobalTransform,
    // pub visibility: Visibility,
    // pub inherited_visibility: InheritedVisibility,
}
impl Piece {
    fn new() -> Self {
        let piece_type: PieceType = PieceType::random();
        Piece {
            piece_type,
            blocks: PieceBlocks(piece_type.get_blocks()),
            position: Position(I16Vec2::new(BOARD_WIDTH as i16 / 2, 0)),
            velocity: Velocity(I16Vec2::new(0, 1)),
            // transform: default(),
            // global_transform: default(),
            // visibility: default(),
            // inherited_visibility: default(),
        }
    }

    fn rotate(&mut self) {
        self.blocks.0 = rotate(self.blocks.0.clone());
    }
}

#[derive(Component)]
struct ActivePieceBlock;

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
        // systems
        .add_systems(Startup, (setup, spawn_piece))
        .run();
}

fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PerfUiCompleteBundle::default());

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -30.0))
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
                    ActivePieceBlock,
                    PbrBundle {
                        mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
                        material: materials.add(piece.piece_type.get_color()),
                        transform: Transform::from_translation(Vec3::new(
                            x as f32,
                            -(y as f32),
                            0.0,
                        )),
                        ..default()
                    },
                ));
            }
        }
    }
}

// fn movement(query: Query<>)