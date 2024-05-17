use bevy::{math::I16Vec2, prelude::*};
use iyes_perf_ui::prelude::*;
use rand::seq::SliceRandom;

const BOARD_WIDTH: i16 = 10;
const BOARD_HEIGHT: i16 = 24;

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
}

#[derive(Component)]
enum PieceRotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}
impl PieceRotation {
    fn new() -> Self {
        PieceRotation::Zero
    }

    fn rotate(&mut self) {
        *self = match self {
            PieceRotation::Zero => PieceRotation::Ninety,
            PieceRotation::Ninety => PieceRotation::OneEighty,
            PieceRotation::OneEighty => PieceRotation::TwoSeventy,
            PieceRotation::TwoSeventy => PieceRotation::Zero,
        }
    }
}

#[derive(Bundle)]
struct Piece {
    piece_type: PieceType,
    rotation: PieceRotation,
    position: Position,
    velocity: Velocity,
}

impl Piece {
    fn new() -> Self {
        Piece {
            piece_type: PieceType::random(),
            rotation: PieceRotation::new(),
            position: Position(I16Vec2::new(BOARD_WIDTH / 2, 0)),
            velocity: Velocity(I16Vec2::new(0, 1)),
        }
    }
}

#[derive(Component)]
struct Position(I16Vec2);

#[derive(Component)]
struct Velocity(pub I16Vec2);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // diagnostics
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        // systems
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PerfUiCompleteBundle::default());

    commands.spawn(Camera3dBundle { ..default() });

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
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..default()
    // });
    // // camera
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });
}
