use crate::rubik::components::{FaceLabel3d, RubikCube};
use bevy::prelude::*;

/// Type alias for the face label update query to avoid type complexity warning
pub type FaceLabelQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Transform, &'static FaceLabel3d),
    (Without<RubikCube>, Without<Camera>),
>;

/// System to update 3D face labels so that they move with the Rubik's cube but remain screen-aligned (billboarded)
pub fn update_face_labels(
    cube_query: Single<&Transform, With<RubikCube>>,
    camera_query: Single<&Transform, With<Camera>>,
    mut label_query: FaceLabelQuery,
) {
    let cube_transform = *cube_query;
    let camera_transform = *camera_query;

    for (mut label_transform, label_info) in &mut label_query {
        let normal = label_info.face.normal();

        // Calculate the world position of the label based on the Rubik's cube's transform
        let local_pos = normal * label_info.dist;
        let world_pos = cube_transform.rotation * local_pos + cube_transform.translation;

        label_transform.translation = world_pos;

        // Keep the labels screen-aligned (billboarded) by matching the camera's rotation
        label_transform.rotation = camera_transform.rotation;
    }
}
