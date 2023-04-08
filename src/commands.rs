use std::sync::Arc;

use bevy_ecs::prelude::*;
use glow::Context;
use tracing::{debug, info, warn};

use crate::components::CustomShader;
use crate::shader::{ShaderBuilder, ShaderType};

/// Despawn an entity and destroy its OpenGL resources
pub fn despawn_and_destroy(entity: Entity, world: &mut World) {
    let gl = world.non_send_resource::<Arc<Context>>().clone();
    if let Some(mut cs) = world.entity_mut(entity).get_mut::<CustomShader>() {
        if let Ok(ref mut shader) = cs.shader {
            unsafe {
                shader.destroy(&gl);
            }
        }
    }
    world.despawn(entity);
}

/// Add a custom shader component to an entity
pub fn add_custom_shader(entity: Entity, world: &mut World) {
    let gl = world.non_send_resource::<Arc<Context>>().clone();
    world.entity_mut(entity).insert(CustomShader::new(&gl));
}

/// Compile the shader in the custom shader component of an entity
pub fn compile_custom_shader(entity: Entity, world: &mut World) {
    let gl = world.non_send_resource::<Arc<Context>>().clone();
    if let Some(mut cs) = world.entity_mut(entity).get_mut::<CustomShader>() {
        // Delete the existing shader program
        if let Ok(ref mut shader) = cs.shader {
            unsafe {
                shader.destroy(&gl);
            }
        }

        cs.shader = ShaderBuilder::new(&gl)
            .add_shader_source(&cs.vert_source, ShaderType::Vertex)
            .and_then(|b| {
                b.add_shader_source(&cs.frag_source, ShaderType::Fragment).and_then(|b| b.link())
            });

        if let Err(e) = &cs.shader {
            warn!("custom shader error: {}", e);
        } else {
            info!("custom shader compilation successful");
        }
    }
}

/// Remove the custom shader component of an entity
pub fn remove_custom_shader(entity: Entity, world: &mut World) {
    let gl = world.non_send_resource::<Arc<Context>>().clone();
    if let Some(mut cs) = world.entity_mut(entity).get_mut::<CustomShader>() {
        if let Ok(ref mut shader) = cs.shader {
            unsafe {
                shader.destroy(&gl);
            }
        }

        world.entity_mut(entity).remove::<CustomShader>();
        debug!("custom shader removed for entity {}", entity.index());
    }
}
