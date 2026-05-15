mod components;
mod input;
mod rubik;

use bevy::prelude::*;
use input::InputPlugin;
use rubik::RubikPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.12))) // Dark premium background
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rubik's Cube ECS - Refactored".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_plugins(RubikPlugin)
        .add_plugins(InputPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_camera_orbit)
        .run();
}

fn setup_scene(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        components::OrbitCamera {
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
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.15,
    });
}

fn update_camera_orbit(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut query: Query<(&mut Transform, &mut components::OrbitCamera)>,
) {
    let Ok((mut transform, mut orbit)) = query.get_single_mut() else {
        return;
    };

    if mouse_button.pressed(MouseButton::Right) {
        for event in mouse_motion.read() {
            orbit.alpha -= event.delta.x * 0.005;
            orbit.beta += event.delta.y * 0.005;
        }
    }

    orbit.beta = orbit.beta.clamp(-1.4, 1.4); // Limit pitch

    let x = orbit.radius * orbit.beta.cos() * orbit.alpha.sin();
    let y = orbit.radius * orbit.beta.sin();
    let z = orbit.radius * orbit.beta.cos() * orbit.alpha.cos();

    transform.translation = Vec3::new(x, y, z);
    transform.look_at(Vec3::ZERO, Vec3::Y);
}
