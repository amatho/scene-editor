use bevy_ecs::prelude::*;

use crate::commands::{AddCustomShader, CompileCustomShader, DespawnMesh};
use crate::components::{CustomShader, Selected};
use crate::resources::{EguiGlowRes, UiState, WinitWindow};
use crate::shader::ShaderType;

pub fn run_ui(
    mut egui_glow: ResMut<EguiGlowRes>,
    window: Res<WinitWindow>,
    mut state: ResMut<UiState>,
    mut selected_entities: Query<(Entity, &Selected, Option<&mut CustomShader>)>,
    all_entities: Query<Entity>,
    mut commands: Commands,
) {
    // Need to reborrow for borrow checker to understand that we borrow different fields
    let state = &mut *state;

    egui_glow.run(&window, |ctx| {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.toggle_value(&mut state.side_panel_open, "🔧 Utilities");
            });
        });

        egui::SidePanel::left("side_panel").show_animated(ctx, state.side_panel_open, |ui| {
            ui.heading("🔧 Utilities");
            if ui.button("Despawn all").clicked() {
                for entity in &all_entities {
                    commands.add(DespawnMesh(entity));
                }
            }
        });

        let selected = selected_entities.get_single_mut();

        if let Ok((entity, _, custom_shader)) = selected {
            match state.editing_mode {
                None => {
                    egui::Window::new("Entity Inspector").show(ctx, |ui| {
                        ui.heading(format!("Entity {}", entity.index()));
                        ui.separator();

                        if ui.button("Edit Vertex Shader").clicked() {
                            state.editing_mode = Some(ShaderType::Vertex);
                        }
                        if ui.button("Edit Fragment Shader").clicked() {
                            state.editing_mode = Some(ShaderType::Fragment);
                        }

                        if ui.button("Despawn").clicked() {
                            commands.add(DespawnMesh(entity));
                        }
                    });
                }
                Some(editing_mode) => {
                    egui::CentralPanel::default().show(ctx, |ui| match custom_shader {
                        Some(mut cs) => {
                            ui.heading(format!("Editing {editing_mode} Shader"));

                            let response = ui.button("Save and close");

                            ui.separator();

                            let shader_source = match editing_mode {
                                ShaderType::Vertex => &mut cs.vert_source,
                                ShaderType::Fragment => &mut cs.frag_source,
                            };

                            ui.add(
                                egui::TextEdit::multiline(shader_source)
                                    .code_editor()
                                    .desired_width(f32::INFINITY),
                            );

                            if response.clicked() {
                                state.editing_mode = None;

                                commands.add(CompileCustomShader(entity));
                            }
                        }
                        None => {
                            commands.add(AddCustomShader(entity));
                        }
                    });
                }
            }
        } else {
            state.editing_mode = None;
        }
    });
}

pub fn paint_ui(mut egui_glow: ResMut<EguiGlowRes>, window: Res<WinitWindow>) {
    egui_glow.paint(&window);
}
