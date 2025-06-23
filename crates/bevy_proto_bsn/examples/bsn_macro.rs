//! BSN macro example
use bevy::{color::palettes::tailwind::*, prelude::*};
use bevy_proto_bsn::{Scene, *};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BsnPlugin)
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
            commands.spawn_scene(ui());
        })
        .run();
}

fn ui() -> impl Scene {
    pbsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(5.0),
        } [
            (Node, {Name::new("BasicButton")}, :button("Basic")),
            (Node, :button("Rounded"), rounded),
            (Node { border: px_all(5.0) }, {BorderColor::all(RED_500.into())} :button("Thick red"), rounded),
            (Node, :button("Merged children"), rounded) [(
                Node {
                    width: px(30.0),
                    height: px(30.0),
                },
                BackgroundColor(BLUE_500),
                {BorderRadius::MAX}
            )],

            (:button("Click me!")) [
                // Observing parent entity
                Obs(|_: On<Pointer<Click>>| {
                    info!("Clicked me!");
                })
            ],

            // Observing entity "BasicButton" by name
            Obs(|_: On<Pointer<Click>>| {
                info!("Clicked Basic!");
            }, @"BasicButton"),
        ]
    }
}

fn button(text: &'static str) -> impl Scene {
    pbsn! {(
        Button,
        Node {
            padding: px_all(5.0),
            border: px_all(2.0),
            align_items: AlignItems::Center,
            column_gap: px(3.0),
        },
        BackgroundColor(LIME_500),
        {BorderColor::all(LIME_800.into())},
    ) [
        ({Text::new(text)}, ConstructTextFont { font: @"Inter-Regular.ttf" })
    ]}
}

fn rounded() -> impl Scene {
    pbsn! {(
        {BorderRadius::all(px(10.0))}
    )}
}
