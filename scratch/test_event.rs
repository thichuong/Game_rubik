use bevy::prelude::*;
fn test_event(mut _reader: EventReader<AppExit>) {}
fn main() {
    App::new().add_systems(Update, test_event);
}
