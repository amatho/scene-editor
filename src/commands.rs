use std::sync::Arc;

use bevy_ecs::prelude::*;
use bevy_ecs::system::Command;
use glow::Context;
use tracing::{info, warn};

use crate::components::{CustomShader, Mesh};
use crate::shader::{ShaderBuilder, ShaderType};

pub struct DespawnMesh(pub Entity);

impl Command for DespawnMesh {
    fn write(self, world: &mut World) {
        let gl = world.non_send_resource::<Arc<Context>>();
        if let Some(mesh) = world.entity(self.0).get::<Mesh>() {
            unsafe {
                mesh.destroy(gl);
            }
        }
        world.despawn(self.0);
    }
}

pub struct AddCustomShader(pub Entity);

impl Command for AddCustomShader {
    fn write(self, world: &mut World) {
        let gl = world.non_send_resource::<Arc<Context>>().clone();
        world.entity_mut(self.0).insert(CustomShader::new(&gl));
    }
}

pub struct CompileCustomShader(pub Entity);

impl Command for CompileCustomShader {
    fn write(self, world: &mut World) {
        let gl = world.non_send_resource::<Arc<Context>>().clone();
        if let Some(mut cs) = world.entity_mut(self.0).get_mut::<CustomShader>() {
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
}
