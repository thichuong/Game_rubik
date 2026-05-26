use bevy::prelude::*;

pub type InteractionQuery<'w, 's, T> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
    (Changed<Interaction>, With<T>),
>;

pub mod app;
pub mod camera;
pub mod environment;
pub mod mapping;
pub mod sidebar;
pub mod size;
pub mod skin;
pub mod solver;

pub use app::*;
pub use camera::*;
pub use environment::*;
pub use mapping::*;
pub use sidebar::*;
pub use size::*;
pub use skin::*;
pub use solver::*;
