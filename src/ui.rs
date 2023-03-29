use std::sync::Arc;

use bevy_ecs::prelude::*;
use glow::Context;
use log::{info, warn};

use crate::components::{CustomShader, Mesh, Selected};
use crate::resources::UiState;
use crate::shader::{ShaderBuilder, ShaderType};

pub fn run_ui(
    gl: NonSend<Arc<Context>>,
    mut state: ResMut<UiState>,
    mut selected_entities: Query<(Entity, &Selected, &Mesh, Option<&mut CustomShader>)>,
    all_entities: Query<(Entity, Option<&Mesh>)>,
    mut commands: Commands,
) {
    // Need to reborrow for borrow checker to understand that we borrow different fields
    let state = &mut *state;

    state.egui_glow.run(&state.window, |ctx| {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.toggle_value(&mut state.side_panel_open, "🔧 Utilities");
            });
        });

        egui::SidePanel::left("side_panel").show_animated(ctx, state.side_panel_open, |ui| {
            ui.heading("🔧 Utilities");
            if ui.button("Despawn all").clicked() {
                for (entity, mesh) in &all_entities {
                    if let Some(mesh) = mesh {
                        unsafe {
                            mesh.destroy(&gl);
                        }
                    }
                    commands.entity(entity).despawn();
                }
            }
        });

        let selected = selected_entities.get_single_mut();

        if let Ok((entity, _, mesh, custom_shader)) = selected {
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
                            unsafe {
                                mesh.destroy(&gl);
                            }
                            commands.entity(entity).despawn();
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

                                cs.shader = ShaderBuilder::new(&gl)
                                    .add_shader_source(&cs.vert_source, ShaderType::Vertex)
                                    .and_then(|b| {
                                        b.add_shader_source(&cs.frag_source, ShaderType::Fragment)
                                            .and_then(|b| b.link())
                                    });

                                if let Err(e) = &cs.shader {
                                    warn!("custom shader error: {}", e);
                                } else {
                                    info!("custom shader compilation successful");
                                }
                            }
                        }
                        None => {
                            commands.entity(entity).insert(CustomShader::new(&gl));
                        }
                    });
                }
            }
        } else {
            state.editing_mode = None;
        }
    });
}

pub fn paint_ui(mut state: ResMut<UiState>) {
    // Need to reborrow for borrow checker to understand that we borrow different fields
    let state = &mut *state;

    state.egui_glow.paint(&state.window);
}
