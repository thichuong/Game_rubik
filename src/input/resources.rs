use bevy::prelude::*;

/// Resource to track mouse drag for rotation
#[derive(Resource, Default)]
pub struct DragState {
    pub start_face: Option<(Entity, Vec3, Vec3)>,
}
