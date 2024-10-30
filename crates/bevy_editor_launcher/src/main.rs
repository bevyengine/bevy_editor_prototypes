//! The launcher for the Bevy Editor.
//!
//! The launcher provide a bunch of functionalities to manage your projects.

use std::path::PathBuf;

use bevy::{
    prelude::*,
    tasks::{block_on, futures_lite::future, IoTaskPool, Task},
};

use bevy_editor::project::{create_new_project, templates::Templates, ProjectInfo};
use bevy_editor_styles::{StylesPlugin, Theme};
use bevy_footer_bar::{FooterBarPlugin, FooterBarSet};
use bevy_scroll_box::ScrollBoxPlugin;
use ui::ProjectList;

mod ui;

/// The Task that creates a new project
#[derive(Component)]
struct CreateProjectTask(Task<std::io::Result<ProjectInfo>>);

/// A utils to run a system only if the [`CreateProjectTask`] is running
fn run_if_task_is_running(task_query: Query<Entity, With<CreateProjectTask>>) -> bool {
    task_query.iter().count() > 0
}

/// Check on the status of the [`CreateProjectTask`] and handle the result when done
fn poll_create_project_task(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut CreateProjectTask)>,
    query: Query<(Entity, &Children), With<ProjectList>>,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    let (task_entity, mut task) = task_query.single_mut();
    if let Some(result) = block_on(future::poll_once(&mut task.0)) {
        match result {
            Ok(project_info) => {
                commands.entity(task_entity).despawn();
                let (project_list_entity, children) = query.iter().next().unwrap();
                let plus_button_entity = children.last().unwrap();
                commands
                    .entity(project_list_entity)
                    .remove_children(&[*plus_button_entity])
                    .with_children(|builder| {
                        ui::spawn_project_node(builder, &theme, &asset_server, &project_info);
                    });
                commands
                    .entity(*plus_button_entity)
                    .set_parent(project_list_entity);
            }
            Err(error) => {
                error!("Failed to create new project: {:?}", error);
                commands.entity(task_entity).despawn();
            }
        }
    }
}

/// Spawn a new [`CreateProjectTask`] to create a new project
fn spawn_create_new_project_task(commands: &mut Commands, template: Templates, path: PathBuf) {
    let task = IoTaskPool::get().spawn(async move { create_new_project(template, path).await });
    commands.spawn_empty().insert(CreateProjectTask(task));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Editor Launcher".to_string(),
                    ..default()
                }),
                ..default()
            }),
            StylesPlugin,
            FooterBarPlugin,
            ScrollBoxPlugin,
        ))
        .add_systems(Startup, ui::setup)
        .add_systems(
            Update,
            poll_create_project_task.run_if(run_if_task_is_running),
        )
        .configure_sets(Startup, FooterBarSet.after(ui::setup))
        .run();
}
