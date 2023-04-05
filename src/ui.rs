use bevy_ecs::prelude::*;
use tracing::warn;

use crate::commands::{AddCustomShader, CompileCustomShader, DespawnMesh};
use crate::components::{CustomShader, Position, Rotation, Scale, Selected, UnloadedMesh};
use crate::resources::{EguiGlowRes, ModelId, ModelLoader, UiState, WinitWindow};
use crate::shader::ShaderType;

type SelectedQuery<'a> = (
    Entity,
    &'a Selected,
    &'a mut Position,
    &'a mut Rotation,
    &'a mut Scale,
    Option<&'a mut CustomShader>,
);

pub fn run_ui(
    mut egui_glow: ResMut<EguiGlowRes>,
    window: Res<WinitWindow>,
    mut state: ResMut<UiState>,
    mut model_loader: ResMut<ModelLoader>,
    mut selected_entities: Query<SelectedQuery>,
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

                egui::SidePanel::right("right_panel").default_width(300.0).show_animated(
                    ctx,
                    selected.is_ok(),
                    |ui| {
                        let Ok((entity, _, mut pos, mut rotation, mut scale, _)) = selected else {
                            unreachable!();
                        };

                        ui.heading("Inspector");
                        ui.strong(format!("Entity {}", entity.index()));
                        ui.separator();

                        egui::Grid::new("inspector_grid").spacing((20.0, 10.0)).show(ui, |ui| {
                            ui.label("Position");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                ui.add(egui::DragValue::new(&mut pos.x).speed(0.1));
                                ui.label("Y:");
                                ui.add(egui::DragValue::new(&mut pos.y).speed(0.1));
                                ui.label("Z:");
                                ui.add(egui::DragValue::new(&mut pos.z).speed(0.1));
                            });
                            ui.end_row();

                            ui.label("Rotation");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                ui.add(egui::DragValue::new(&mut rotation.x).speed(0.1));
                                ui.label("Y:");
                                ui.add(egui::DragValue::new(&mut rotation.y).speed(0.1));
                                ui.label("Z:");
                                ui.add(egui::DragValue::new(&mut rotation.z).speed(0.1));
                            });
                            ui.end_row();

                            ui.label("Scale");
                            ui.horizontal(|ui| {
                                ui.label("X:");
                                ui.add(egui::DragValue::new(&mut scale.x).speed(0.1));
                                ui.label("Y:");
                                ui.add(egui::DragValue::new(&mut scale.y).speed(0.1));
                                ui.label("Z:");
                                ui.add(egui::DragValue::new(&mut scale.z).speed(0.1));
                            });
                            ui.end_row();

                            ui.horizontal(|_| {});
                            if ui.button("Reset Transform").clicked() {
                                *pos = Default::default();
                                *rotation = Default::default();
                                *scale = Default::default();
                            }
                            ui.end_row();

                            ui.label("Custom Shader");
                            ui.vertical(|ui| {
                                if ui.button("Edit Vertex").clicked() {
                                    state.editing_mode = Some(ShaderType::Vertex);
                                }
                                if ui.button("Edit Fragment").clicked() {
                                    state.editing_mode = Some(ShaderType::Fragment);
                                }
                                if ui.button("Reset Shaders").clicked() {
                                    commands.entity(entity).remove::<CustomShader>();
                                }
                            });
                            ui.end_row();

                            ui.label("Change Model");
                            ui.vertical(|ui| {
                                egui::ComboBox::from_id_source("model_select")
                                    .selected_text(format!("{:?}", state.selected_model))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut state.selected_model,
                                            ModelId::Cube,
                                            "Cube",
                                        );
                                        ui.selectable_value(
                                            &mut state.selected_model,
                                            ModelId::Plane,
                                            "Plane",
                                        );
                                    });

                                if ui.button("Load").clicked() {
                                    if let Ok(model) = model_loader.load_model(state.selected_model)
                                    {
                                        commands.entity(entity).insert(UnloadedMesh::from(model));
                                    } else {
                                        warn!("could not load model {:?}", state.selected_model);
                                    }
                                }
                            });
                            ui.end_row();

                            ui.label("Commands");
                            if ui.button("Despawn").clicked() {
                                commands.add(DespawnMesh(entity));
                            }
                            ui.end_row();
                        });
                    },
                );
            }
            Some(editing_mode) => {
                if let Ok((entity, _, _, _, _, custom_shader)) = selected {
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
