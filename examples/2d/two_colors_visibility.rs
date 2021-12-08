use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*
};
use bevy_svg::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "two_colors_visibility".to_string(),
            width: 400.0,
            height: 400.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_svg::prelude::SvgPlugin)
        .add_startup_system(setup)
        .add_system(keyboard_input_system)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let svg = asset_server.load("neutron_star.svg");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SvgBundle {
        svg,
        origin: Origin::Center,
        visible: Visible { is_visible: false, is_transparent: true },
        ..Default::default()
    });
}

/// This system toggles SVG visibility when 'V' is pressed
fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<
        (&Handle<Svg>, &mut Visible),
    >,
) {
    if keyboard_input.just_pressed(KeyCode::V) {
        for (_, mut visible) in query.iter_mut() {
            visible.is_visible = !visible.is_visible;
        }
    }
}
