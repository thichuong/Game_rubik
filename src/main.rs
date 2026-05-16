mod camera;
mod events;
mod input;
mod rubik;
mod solver;
mod ui;

use bevy::prelude::*;
use camera::components::OrbitCamera;
use camera::CameraPlugin;
use events::{ResetCameraEvent, SolveByStepsEvent};
use input::InputPlugin;
use rubik::RubikPlugin;
use solver::SolverPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 1.0))))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rubik's Cube ECS - Modular Refactor".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_message::<ResetCameraEvent>()
        .add_message::<SolveByStepsEvent>()
        .add_plugins(CameraPlugin)
        .add_plugins(RubikPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(SolverPlugin)
        .add_plugins(UiPlugin)
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            radius: 10.0,
            alpha: 0.785, // 45 degrees
            beta: 0.785,  // 45 degrees
        },
    ));

    // Studio Lighting
    // Main key light
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: true,
            range: 30.0,
            ..default()
        },
        Transform::from_xyz(5.0, 8.0, 5.0),
    ));

    // Fill light
    commands.spawn((
        PointLight {
            intensity: 1_000_000.0,
            range: 20.0,
            ..default()
        },
        Transform::from_xyz(-5.0, 4.0, -5.0),
    ));

    // Rim light from bottom
    commands.spawn((
        PointLight {
            intensity: 800_000.0,
            range: 20.0,
            ..default()
        },
        Transform::from_xyz(0.0, -8.0, 0.0),
    ));

    // Ambient light for soft shadows
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 0.15,
        affects_lightmapped_meshes: false,
    });
}
