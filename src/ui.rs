use std::sync::Arc;

use bevy_ecs::prelude::*;
use glow::Context;

use crate::components::{Mesh, Selected};
use crate::resources::{EguiGlowRes, WinitWindow};

pub fn run_ui(
    gl: NonSend<Arc<Context>>,
    mut egui_glow: ResMut<EguiGlowRes>,
    window: Res<WinitWindow>,
    selected_entities: Query<(Entity, &Selected, Option<&Mesh>)>,
    all_entities: Query<(Entity, Option<&Mesh>)>,
    mut commands: Commands,
) {
    egui_glow.run(&window, |ctx| {
        let selected = selected_entities.get_single();

        egui::SidePanel::left("my_side_panel").show(ctx, |ui| {
            ui.heading("Hello World!");
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

        if let Ok((entity, _, mesh)) = selected {
            egui::Window::new("Selected Entity").show(ctx, |ui| {
                ui.label(format!("Entity {}", entity.index()));
                if ui.button("Despawn").clicked() {
                    if let Some(mesh) = mesh {
                        unsafe {
                            mesh.destroy(&gl);
                        }
                    }
                    commands.entity(entity).despawn();
                }
            });
        }
    });
}

pub fn paint_ui(mut egui_glow: ResMut<EguiGlowRes>, window: Res<WinitWindow>) {
    egui_glow.paint(&window);
}
