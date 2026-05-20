use bevy::prelude::*;
use bevy_resvg::prelude::*;
use game_rubik::camera::CameraPlugin;
use game_rubik::camera::components::OrbitCamera;
use game_rubik::environment::EnvironmentPlugin;
use game_rubik::events::{ResetCameraEvent, SolveByStepsEvent};
use game_rubik::input::InputPlugin;
use game_rubik::rubik::RubikPlugin;
use game_rubik::ui::UiPlugin;
use rubik_solver::SolverPlugin;

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
