use crate::environment::resources::{EnvironmentSettings, LightRig, MainLight};
use bevy::prelude::*;

pub fn setup_environment(
    mut commands: Commands,
    settings: Res<EnvironmentSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Light Rig (container for directional lights to allow rotation)
    commands
        .spawn((Transform::IDENTITY, Visibility::default(), LightRig))
        .with_children(|parent| {
            // Main key light
            parent.spawn((
                PointLight {
                    intensity: settings.light_intensity,
                    shadows_enabled: true,
                    range: 30.0,
                    color: settings.color_temperature,
                    ..default()
                },
                Transform::from_xyz(5.0, 8.0, 5.0),
                MainLight,
            ));

            // Fill light
            parent.spawn((
                PointLight {
                    intensity: settings.light_intensity * 0.5,
                    range: 20.0,
                    color: settings.color_temperature,
                    ..default()
                },
                Transform::from_xyz(-5.0, 4.0, -5.0),
            ));

            // Rim light from bottom
            parent.spawn((
                PointLight {
                    intensity: settings.light_intensity * 0.4,
                    range: 20.0,
                    color: settings.color_temperature,
                    ..default()
                },
                Transform::from_xyz(0.0, -8.0, 0.0),
            ));
        });

    // Floor plane to receive shadows
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: settings.background_color,
            perceptual_roughness: 0.9,
            reflectance: 0.1,
            ..default()
        })),
        Transform::from_xyz(0.0, -2.5, 0.0),
        crate::environment::resources::Floor,
    ));

    // Ambient light
    commands.insert_resource(GlobalAmbientLight {
        color: settings.color_temperature,
        brightness: settings.ambient_brightness,
        ..default()
    });

    // Background color
    commands.insert_resource(ClearColor(settings.background_color));
}

pub fn update_environment(
    settings: Res<EnvironmentSettings>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut clear_color: ResMut<ClearColor>,
    mut light_query: Query<&mut PointLight>,
    mut rig_query: Single<&mut Transform, With<LightRig>>,
    floor_material: Single<&MeshMaterial3d<StandardMaterial>, With<crate::environment::resources::Floor>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if settings.is_changed() {
        // Update ambient
        ambient_light.brightness = settings.ambient_brightness;
        ambient_light.color = settings.color_temperature;

        // Update clear color
        clear_color.0 = settings.background_color;

        // Update lights
        for mut light in &mut light_query {
            light.color = settings.color_temperature;
        }

        // Update floor color to match background
        if let Some(material) = materials.get_mut(&floor_material.0) {
            material.base_color = settings.background_color;
        }

        rig_query.rotation = Quat::from_rotation_y(settings.light_angle);
    }
}

pub fn update_light_intensity(
    settings: Res<EnvironmentSettings>,
    mut main_light: Single<&mut PointLight, With<MainLight>>,
) {
    if settings.is_changed() {
        main_light.intensity = settings.light_intensity;
    }
}
