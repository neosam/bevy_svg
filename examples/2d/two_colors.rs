use bevy::prelude::*;
use bevy_svg::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "two_colors".to_string(),
            width: 400.0,
            height: 400.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_svg::prelude::SvgPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SvgBuilder::from_file("examples/assets/neutron_star.svg")
            .origin(Origin::Center)
            .position(Vec3::new(0.0, 0.0, 0.0))
            .build()
            .unwrap()
        );
}
