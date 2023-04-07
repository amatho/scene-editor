use std::borrow::Cow;
use std::sync::Arc;

use bevy_ecs::prelude::*;
use bevy_ecs::system::Command;
use glow::Context;
use tracing::{debug, info, warn};

use crate::components::{CustomShader, Mesh};
use crate::resources::ModelLoader;
use crate::shader::{ShaderBuilder, ShaderType};

pub struct LoadMesh {
    entity: Entity,
    model_name: Cow<'static, str>,
}

impl LoadMesh {
    pub fn new<T>(entity: Entity, model_name: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self { entity, model_name: model_name.into() }
    }
}

impl Command for LoadMesh {
    fn write(self, world: &mut World) {
        let gl = world.non_send_resource::<Arc<Context>>().clone();
        let model_loader = world.resource::<ModelLoader>();

        if let Some(tobj_mesh) = model_loader.get(&self.model_name) {
            let mesh = Mesh::from_tobj_mesh(&gl, tobj_mesh);
            let mut entity_mut = world.entity_mut(self.entity);

            // Clean up after old mesh
            if let Some(mut mesh) = entity_mut.get_mut::<Mesh>() {
                unsafe {
                    mesh.destroy(&gl);
                }

                entity_mut.remove::<Mesh>();
            }

            entity_mut.insert(mesh);
        } else {
            warn!("could not load model {:?}", self.model_name);
        }
    }
}

/// Despawn an entity and destroy its OpenGL resources
pub fn despawn_and_destroy(entity: Entity, world: &mut World) {
    let gl = world.non_send_resource::<Arc<Context>>().clone();
    if let Some(mut mesh) = world.entity_mut(entity).get_mut::<Mesh>() {
        unsafe {
            mesh.destroy(&gl);
        }
    }
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
