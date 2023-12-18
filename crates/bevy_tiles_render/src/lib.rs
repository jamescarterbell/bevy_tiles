use bevy::{
    app::Plugin,
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::Transparent2d,
    ecs::schedule::{apply_deferred, IntoSystemConfigs},
    render::{
        render_phase::AddRenderCommand,
        render_resource::{Shader, SpecializedRenderPipelines},
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use chunk::internal::SavedChunks;
use cleanup::save_chunks;
use extract::extract_chunks;
use maps::internal::SavedMaps;
use prepare::{create_bind_groups, prepare_chunk_batch, prepare_chunks};
use queue::{create_chunk_batches, queue_chunks};

use crate::{draw::DrawChunks, pipeline::TilesChunkPipeline};

mod bindings;
mod buffer_helpers;
pub mod chunk;
mod cleanup;
mod draw;
mod extract;
pub mod maps;
mod pipeline;
mod prepare;
mod queue;
pub mod tiles;

const TILES_VERT: Handle<Shader> = Handle::weak_from_u128(163058266501073814892310220797241232500);
const TILES_FRAG: Handle<Shader> = Handle::weak_from_u128(163058266501073814892310220797241232501);

pub struct TilesRenderPlugin;

impl Plugin for TilesRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_app = app.get_sub_app_mut(RenderApp).expect("No RenderApp found!");

        render_app.init_resource::<SavedMaps>();
        render_app.init_resource::<SavedChunks>();

        // Respawn chunks that we saved from the last frame
        // Copy over tile data
        render_app
            .add_systems(ExtractSchedule, extract_chunks)
            .add_systems(
                Render,
                (create_chunk_batches, apply_deferred, queue_chunks)
                    .chain()
                    .in_set(RenderSet::Queue),
            )
            .add_systems(
                Render,
                (
                    prepare_chunks,
                    apply_deferred,
                    prepare_chunk_batch,
                    apply_deferred,
                    create_bind_groups,
                )
                    .chain()
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, (save_chunks).in_set(RenderSet::Cleanup));
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        let render_app = app.get_sub_app_mut(RenderApp).expect("No RenderApp found!");

        render_app.add_render_command::<Transparent2d, DrawChunks>();
        render_app
            .init_resource::<TilesChunkPipeline>()
            .init_resource::<SpecializedRenderPipelines<TilesChunkPipeline>>();

        load_internal_asset!(
            app,
            TILES_FRAG,
            "shaders/tiles_frag.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            TILES_VERT,
            "shaders/tiles_vert.wgsl",
            Shader::from_wgsl
        );
    }
}
