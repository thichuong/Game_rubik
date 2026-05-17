use crate::rubik::resources::{RubikMaterials, RubikSkin, SkinType};
use bevy::prelude::*;

pub fn update_skins(
    skin: Res<RubikSkin>,
    rubik_materials: Res<RubikMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !skin.is_changed() {
        return;
    }

    let texture = match skin.current {
        SkinType::Classic => None,
        SkinType::Carbon => Some(rubik_materials.carbon_tex.clone()),
        SkinType::Geometric => Some(rubik_materials.geometric_tex.clone()),
        SkinType::Floral => Some(rubik_materials.floral_tex.clone()),
    };

    let face_materials = [
        &rubik_materials.white,
        &rubik_materials.yellow,
        &rubik_materials.red,
        &rubik_materials.orange,
        &rubik_materials.green,
        &rubik_materials.blue,
    ];

    for handle in face_materials {
        if let Some(mat) = materials.get_mut(handle) {
            mat.base_color_texture.clone_from(&texture);
        }
    }
}
