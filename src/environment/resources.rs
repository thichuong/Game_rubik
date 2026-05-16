use bevy::prelude::*;

#[derive(Resource)]
pub struct EnvironmentSettings {
    pub ambient_brightness: f32,
    pub light_intensity: f32,
    pub color_temperature: Color,
    pub light_angle: f32, // in radians
    pub background_color: Color,
}

impl Default for EnvironmentSettings {
    fn default() -> Self {
        Self {
            ambient_brightness: 0.15,
            light_intensity: 1_000_000.0,
            color_temperature: Color::WHITE,
            light_angle: 0.0,
            background_color: Color::Srgba(Srgba::new(0.1, 0.1, 0.12, 1.0)),
        }
    }
}

#[derive(Component)]
pub struct MainLight;

#[derive(Component)]
pub struct LightRig;

#[derive(Component)]
pub struct Floor;
