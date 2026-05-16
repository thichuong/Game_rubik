mod camera;
mod environment;
mod events;
mod input;
mod rubik;
mod solver;
mod ui;

use bevy::prelude::*;
use bevy_resvg::prelude::*;
use camera::components::OrbitCamera;
use camera::CameraPlugin;
use environment::EnvironmentPlugin;
use events::{ResetCameraEvent, SolveByStepsEvent};
use input::InputPlugin;
use rubik::RubikPlugin;
use solver::SolverPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
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
        .add_plugins(EnvironmentPlugin)
        .add_plugins(RubikPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(SolverPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(SvgPlugin)
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
}
