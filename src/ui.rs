use bevy::prelude::*;

use bevy_egui::EguiContexts;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use bevy_mesh_terrain::edit::{BrushType, TerrainCommandEvent};

use std::fmt::{self, Display, Formatter};

use crate::editor_pls::bevy_pls_editor_is_active;

pub fn editor_ui_plugin(app: &mut App) {
    app.init_resource::<EditorToolsState>()
        .add_plugins(EguiPlugin)
        .add_systems(Update, editor_tools.run_if( not( bevy_pls_editor_is_active ) )  );
}

#[derive(Default, Resource, Clone)]
pub struct LinearPixelColor {
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub a: u16,
}

#[derive(Default, Resource, Clone)]
pub struct EditorToolsState {
    pub tool_mode: ToolMode,
    pub brush_type: BrushType,
    pub brush_radius: u32,
    pub brush_hardness: u32,
    pub color: LinearPixelColor, //brush mode
}
/*
impl Default for EditorToolsState{

    fn default() -> Self{
        Self{

            brush_radius: 50,
            brush_hardness: 100,
            ..default()
        }
        }
    }
    */

#[derive(Eq, PartialEq, Debug, Default, Clone)]
pub enum ToolMode {
    #[default]
    Height,
    Splat,
}
const TOOL_MODES: [ToolMode; 2] = [ToolMode::Height, ToolMode::Splat];

const BRUSH_TYPES: [BrushType; 3] = [BrushType::SetExact, BrushType::Smooth, BrushType::Noise];

impl Display for ToolMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let label = match self {
            ToolMode::Height => "Height",
            ToolMode::Splat => "Splat",
        };

        write!(f, "{}", label)
    }
}



fn editor_tools(

 

    mut tools_state: ResMut<EditorToolsState>,

    mut command_event_writer: EventWriter<TerrainCommandEvent>,

    mut contexts: EguiContexts,
) {


 


    egui::Window::new("Editor Tools").show(contexts.ctx_mut(), |ui| {
        if ui.button("Save All Chunks (Ctrl+S)").clicked() {
            command_event_writer.send(TerrainCommandEvent::SaveAllChunks(true, true, true));
        }

        if ui.button("Save Splat and Height").clicked() {
            command_event_writer.send(TerrainCommandEvent::SaveAllChunks(true, true, false));
        }

        ui.spacing();
        ui.separator();

        /*ui.horizontal(|ui| {
            let name_label = ui.label("Your name: ");
            ui.text_edit_singleline(&mut tools_state.name)
                .labelled_by(name_label.id);
        });*/

        ui.heading("Tool Mode");
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.spacing();
            egui::ComboBox::new("tool_mode", "")
                .selected_text(tools_state.tool_mode.to_string())
                .show_ui(ui, |ui| {
                    for tool_mode in TOOL_MODES.into_iter() {
                        if ui
                            .selectable_label(
                                tools_state.tool_mode == tool_mode,
                                tool_mode.to_string(),
                            )
                            .clicked()
                        {
                            tools_state.tool_mode = tool_mode;
                        }
                    }
                });
        });
        ui.spacing();
        ui.separator();

        ui.add(egui::Slider::new(&mut tools_state.brush_radius, 0..=100).text("Brush Radius"));
        ui.add(egui::Slider::new(&mut tools_state.brush_hardness, 0..=100).text("Brush Hardness"));

        match tools_state.tool_mode {
            ToolMode::Splat => {
                ui.add(
                    egui::Slider::new(&mut tools_state.color.r, 0..=255)
                        .text("Texture A (R_Channel"),
                );
                ui.add(
                    egui::Slider::new(&mut tools_state.color.g, 0..=255)
                        .text("Texture B (G_Channel"),
                );
                ui.add(
                    egui::Slider::new(&mut tools_state.color.b, 0..=255)
                        .text("Layer Fade (B_Channel"),
                );
            }
            ToolMode::Height => {
                egui::ComboBox::new("brush_type", "")
                    .selected_text(tools_state.brush_type.to_string())
                    .show_ui(ui, |ui| {
                        for brush_type in BRUSH_TYPES.into_iter() {
                            if ui
                                .selectable_label(
                                    tools_state.brush_type == brush_type,
                                    brush_type.to_string(),
                                )
                                .clicked()
                            {
                                tools_state.brush_type = brush_type;
                            }
                        }
                    });

                ui.add(
                    egui::Slider::new(&mut tools_state.color.r, 0..=65535)
                        .text("Height (R_Channel)"),
                );
            }
        }
    });
}
