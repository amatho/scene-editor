use bevy_ecs::prelude::*;

use crate::commands::{AddCustomShader, CompileCustomShader, DespawnMesh};
use crate::components::{CustomShader, Position, Rotation, Selected};
use crate::resources::{EguiGlowRes, UiState, WinitWindow};
use crate::shader::ShaderType;

pub fn run_ui(
    mut egui_glow: ResMut<EguiGlowRes>,
    window: Res<WinitWindow>,
    mut state: ResMut<UiState>,
    mut selected_entities: Query<(
        Entity,
        &Selected,
        &mut Position,
        &mut Rotation,
        Option<&mut CustomShader>,
    )>,
    all_entities: Query<Entity>,
    mut commands: Commands,
) {
    // Need to reborrow for borrow checker to understand that we borrow different fields
    let state = &mut *state;

    egui_glow.run(&window, |ctx| {
        let selected = selected_entities.get_single_mut();

        match state.editing_mode {
            None => {
                egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.toggle_value(&mut state.utilities_open, "🔧 Utilities");
                    });
                });

                egui::SidePanel::left("left_panel").show_animated(
                    ctx,
                    state.utilities_open,
                    |ui| {
                        ui.heading("🔧 Utilities");
                        if ui.button("Despawn all").clicked() {
                            for entity in &all_entities {
                                commands.add(DespawnMesh(entity));
                            }
                        }
                    },
                );

                if let Ok((entity, _, mut pos, mut rotation, _)) = selected {
                    egui::SidePanel::right("right_panel").default_width(300.0).show(ctx, |ui| {
                        ui.heading(format!("Entity {}", entity.index()));
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label("Position");
                            ui.add_space(1.0);
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut pos.x).speed(0.1));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut pos.y).speed(0.1));
                            ui.label("Z:");
                            ui.add(egui::DragValue::new(&mut pos.z).speed(0.1));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Rotation");
                            ui.add_space(1.0);
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut rotation.x).speed(0.1));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut rotation.y).speed(0.1));
                            ui.label("Z:");
                            ui.add(egui::DragValue::new(&mut rotation.z).speed(0.1));
                        });

                        ui.add_space(10.0);

                        ui.label("Edit custom shader:");
                        ui.horizontal(|ui| {
                            if ui.button("Vertex Shader").clicked() {
                                state.editing_mode = Some(ShaderType::Vertex);
                            }
                            if ui.button("Fragment Shader").clicked() {
                                state.editing_mode = Some(ShaderType::Fragment);
                            }
                        });

                        ui.add_space(10.0);

                        if ui.button("Despawn").clicked() {
                            commands.add(DespawnMesh(entity));
                        }
                    });
                } else {
                    state.editing_mode = None;
                }
            }
            Some(editing_mode) => {
                if let Ok((entity, _, _, _, custom_shader)) = selected {
                    match custom_shader {
                        Some(mut cs) => {
                            egui::CentralPanel::default().show(ctx, |ui| {
                                ui.heading(format!("Editing {editing_mode} Shader"));
                                let response = ui.button("Save and close");
                                ui.separator();

                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    let shader_source = match editing_mode {
                                        ShaderType::Vertex => &mut cs.vert_source,
                                        ShaderType::Fragment => &mut cs.frag_source,
                                    };

                                    ui.add(
                                        egui::TextEdit::multiline(shader_source)
                                            .code_editor()
                                            .desired_width(f32::INFINITY),
                                    );
                                });

                                if response.clicked() {
                                    state.editing_mode = None;

                                    commands.add(CompileCustomShader(entity));
                                }
                            });
                        }
                        None => {
                            commands.add(AddCustomShader(entity));
                        }
                    }
                }
            }
        }
    });
}

pub fn paint_ui(mut egui_glow: ResMut<EguiGlowRes>, window: Res<WinitWindow>) {
    egui_glow.paint(&window);
}
