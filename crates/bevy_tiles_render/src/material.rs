use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        extract_component::ExtractComponentPlugin,
        globals::GlobalsBuffer,
        render_asset::{PrepareAssetSet, RenderAssets},
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase},
        render_resource::{
            AsBindGroup, AsBindGroupError, BindGroup, BindGroupDescriptor, BindGroupEntry,
            BindGroupLayout, BindingResource, OwnedBindingResource, PipelineCache,
            RenderPipelineDescriptor, ShaderRef, SpecializedRenderPipeline,
            SpecializedRenderPipelines,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
        view::{ExtractedView, ViewUniforms, VisibleEntities},
        Extract, Render, RenderApp, RenderSet,
    },
    utils::{FloatOrd, HashMap, HashSet},
};
use std::{hash::Hash, marker::PhantomData};

#[cfg(not(feature = "atlas"))]
use bevy::render::renderer::RenderQueue;

use crate::prelude::TilemapId;

use super::{
    chunk::{ChunkId, RenderChunk2dStorage},
    draw::DrawTilemapMaterial,
    pipeline::{TilemapPipeline, TilemapPipelineKey},
    queue::{ImageBindGroups, TilemapViewBindGroup},
    RenderYSort,
};

#[cfg(not(feature = "atlas"))]
pub(crate) use super::TextureArrayCache;

pub trait MaterialTilemap:
    AsBindGroup + Send + Sync + Clone + TypeUuid + TypePath + Sized + 'static
{
    /// Returns this material's vertex shader. If [`ShaderRef::Default`] is returned, the default mesh vertex shader
    /// will be used.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Returns this material's fragment shader. If [`ShaderRef::Default`] is returned, the default mesh fragment shader
    /// will be used.
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }

    /// Customizes the default [`RenderPipelineDescriptor`].
    #[allow(unused_variables)]
    #[inline]
    fn specialize(descriptor: &mut RenderPipelineDescriptor, key: MaterialTilemapKey<Self>) {}
}

pub struct MaterialTilemapKey<M: MaterialTilemap> {
    pub tilemap_pipeline_key: TilemapPipelineKey,
    pub bind_group_data: M::Data,
}

impl<M: MaterialTilemap> Eq for MaterialTilemapKey<M> where M::Data: PartialEq {}

impl<M: MaterialTilemap> PartialEq for MaterialTilemapKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.tilemap_pipeline_key == other.tilemap_pipeline_key
            && self.bind_group_data == other.bind_group_data
    }
}

impl<M: MaterialTilemap> Clone for MaterialTilemapKey<M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            tilemap_pipeline_key: self.tilemap_pipeline_key,
            bind_group_data: self.bind_group_data.clone(),
        }
    }
}

impl<M: MaterialTilemap> Hash for MaterialTilemapKey<M>
where
    M::Data: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tilemap_pipeline_key.hash(state);
        self.bind_group_data.hash(state);
    }
}

pub struct MaterialTilemapPlugin<M: MaterialTilemap>(PhantomData<M>);

impl<M: MaterialTilemap> Default for MaterialTilemapPlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: MaterialTilemap> Plugin for MaterialTilemapPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_asset::<M>()
            .add_plugins(ExtractComponentPlugin::<Handle<M>>::extract_visible());
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent2d, DrawTilemapMaterial<M>>()
                .init_resource::<MaterialTilemapPipeline<M>>()
                .init_resource::<ExtractedMaterialsTilemap<M>>()
                .init_resource::<RenderMaterialsTilemap<M>>()
                .init_resource::<SpecializedRenderPipelines<MaterialTilemapPipeline<M>>>()
                .add_systems(ExtractSchedule, extract_materials_tilemap::<M>)
                .add_systems(
                    Render,
                    prepare_materials_tilemap::<M>
                        .in_set(RenderSet::Prepare)
                        .after(PrepareAssetSet::PreAssetPrepare),
                )
                .add_systems(
                    Render,
                    queue_material_tilemap_meshes::<M>.in_set(RenderSet::Queue),
                );
        }
    }
}

pub struct PreparedMaterialTilemap<T: MaterialTilemap> {
    pub bindings: Vec<OwnedBindingResource>,
    pub bind_group: BindGroup,
    pub key: T::Data,
}

#[derive(Resource)]
struct ExtractedMaterialsTilemap<M: MaterialTilemap> {
    extracted: Vec<(Handle<M>, M)>,
    removed: Vec<Handle<M>>,
}

impl<M: MaterialTilemap> Default for ExtractedMaterialsTilemap<M> {
    fn default() -> Self {
        Self {
            extracted: Default::default(),
            removed: Default::default(),
        }
    }
}

#[derive(Resource)]
pub struct MaterialTilemapPipeline<M: MaterialTilemap> {
    pub tilemap_pipeline: TilemapPipeline,
    pub material_tilemap_layout: BindGroupLayout,
    pub vertex_shader: Option<Handle<Shader>>,
    pub fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<M>,
}

impl<M: MaterialTilemap> Clone for MaterialTilemapPipeline<M> {
    fn clone(&self) -> Self {
        Self {
            tilemap_pipeline: self.tilemap_pipeline.clone(),
            material_tilemap_layout: self.material_tilemap_layout.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            marker: PhantomData,
        }
    }
}

impl<M: MaterialTilemap> SpecializedRenderPipeline for MaterialTilemapPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = MaterialTilemapKey<M>;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut descriptor = self.tilemap_pipeline.specialize(key.tilemap_pipeline_key);
        if let Some(vertex_shader) = &self.vertex_shader {
            descriptor.vertex.shader = vertex_shader.clone();
        }

        if let Some(fragment_shader) = &self.fragment_shader {
            descriptor.fragment.as_mut().unwrap().shader = fragment_shader.clone();
        }
        descriptor.layout = vec![
            self.tilemap_pipeline.view_layout.clone(),
            self.tilemap_pipeline.mesh_layout.clone(),
            self.tilemap_pipeline.material_layout.clone(),
            self.material_tilemap_layout.clone(),
        ];

        M::specialize(&mut descriptor, key);
        descriptor
    }
}

impl<M: MaterialTilemap> FromWorld for MaterialTilemapPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();
        let material_tilemap_layout = M::bind_group_layout(render_device);

        MaterialTilemapPipeline {
            tilemap_pipeline: world.resource::<TilemapPipeline>().clone(),
            material_tilemap_layout,
            vertex_shader: match M::vertex_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            fragment_shader: match M::fragment_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            marker: PhantomData,
        }
    }
}

/// Stores all prepared representations of [`Material2d`] assets for as long as they exist.
#[derive(Resource, Deref, DerefMut)]
pub struct RenderMaterialsTilemap<T: MaterialTilemap>(
    HashMap<Handle<T>, PreparedMaterialTilemap<T>>,
);

impl<T: MaterialTilemap> Default for RenderMaterialsTilemap<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// This system extracts all created or modified assets of the corresponding [`Material2d`] type
/// into the "render world".
fn extract_materials_tilemap<M: MaterialTilemap>(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<M>>>,
    assets: Extract<Res<Assets<M>>>,
) {
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                changed_assets.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed_assets.remove(handle);
                removed.push(handle.clone_weak());
            }
        }
    }

    let mut extracted_assets = Vec::new();
    for handle in changed_assets.drain() {
        if let Some(asset) = assets.get(&handle) {
            extracted_assets.push((handle, asset.clone()));
        }
    }

    commands.insert_resource(ExtractedMaterialsTilemap {
        extracted: extracted_assets,
        removed,
    });
}

/// All [`Material2d`] values of a given type that should be prepared next frame.
pub struct PrepareNextFrameMaterials<M: MaterialTilemap> {
    assets: Vec<(Handle<M>, M)>,
}

impl<M: MaterialTilemap> Default for PrepareNextFrameMaterials<M> {
    fn default() -> Self {
        Self {
            assets: Default::default(),
        }
    }
}

/// This system prepares all assets of the corresponding [`Material2d`] type
/// which where extracted this frame for the GPU.
fn prepare_materials_tilemap<M: MaterialTilemap>(
    mut prepare_next_frame: Local<PrepareNextFrameMaterials<M>>,
    mut extracted_assets: ResMut<ExtractedMaterialsTilemap<M>>,
    mut render_materials: ResMut<RenderMaterialsTilemap<M>>,
    render_device: Res<RenderDevice>,
    images: Res<RenderAssets<Image>>,
    fallback_image: Res<FallbackImage>,
    pipeline: Res<MaterialTilemapPipeline<M>>,
) {
    let queued_assets = std::mem::take(&mut prepare_next_frame.assets);
    for (handle, material) in queued_assets {
        match prepare_material_tilemap(
            &material,
            &render_device,
            &images,
            &fallback_image,
            &pipeline,
        ) {
            Ok(prepared_asset) => {
                render_materials.insert(handle, prepared_asset);
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                prepare_next_frame.assets.push((handle, material));
            }
        }
    }

    for removed in std::mem::take(&mut extracted_assets.removed) {
        render_materials.remove(&removed);
    }

    for (handle, material) in std::mem::take(&mut extracted_assets.extracted) {
        match prepare_material_tilemap(
            &material,
            &render_device,
            &images,
            &fallback_image,
            &pipeline,
        ) {
            Ok(prepared_asset) => {
                render_materials.insert(handle, prepared_asset);
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                prepare_next_frame.assets.push((handle, material));
            }
        }
    }
}

fn prepare_material_tilemap<M: MaterialTilemap>(
    material: &M,
    render_device: &RenderDevice,
    images: &RenderAssets<Image>,
    fallback_image: &FallbackImage,
    pipeline: &MaterialTilemapPipeline<M>,
) -> Result<PreparedMaterialTilemap<M>, AsBindGroupError> {
    let prepared = material.as_bind_group(
        &pipeline.material_tilemap_layout,
        render_device,
        images,
        fallback_image,
    )?;
    Ok(PreparedMaterialTilemap {
        bindings: prepared.bindings,
        bind_group: prepared.bind_group,
        key: prepared.data,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn queue_material_tilemap_meshes<M: MaterialTilemap>(
    mut commands: Commands,
    y_sort: Res<RenderYSort>,
    chunk_storage: Res<RenderChunk2dStorage>,
    transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
    render_device: Res<RenderDevice>,
    (tilemap_pipeline, material_tilemap_pipeline, mut material_pipelines): (
        Res<TilemapPipeline>,
        Res<MaterialTilemapPipeline<M>>,
        ResMut<SpecializedRenderPipelines<MaterialTilemapPipeline<M>>>,
    ),
    pipeline_cache: Res<PipelineCache>,
    view_uniforms: Res<ViewUniforms>,
    gpu_images: Res<RenderAssets<Image>>,
    msaa: Res<Msaa>,
    globals_buffer: Res<GlobalsBuffer>,
    mut image_bind_groups: ResMut<ImageBindGroups>,
    (standard_tilemap_meshes, materials): (
        Query<(Entity, &ChunkId, &Transform, &TilemapId)>,
        Query<&Handle<M>>,
    ),
    mut views: Query<(
        Entity,
        &ExtractedView,
        &VisibleEntities,
        &mut RenderPhase<Transparent2d>,
    )>,
    render_materials: Res<RenderMaterialsTilemap<M>>,
    #[cfg(not(feature = "atlas"))] (mut texture_array_cache, render_queue): (
        ResMut<TextureArrayCache>,
        Res<RenderQueue>,
    ),
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    #[cfg(not(feature = "atlas"))]
    texture_array_cache.queue(&render_device, &render_queue, &gpu_images);

    if standard_tilemap_meshes.is_empty() {
        return;
    }

    if let (Some(view_binding), Some(globals)) = (
        view_uniforms.uniforms.binding(),
        globals_buffer.buffer.binding(),
    ) {
        for (entity, view, visible_entities, mut transparent_phase) in views.iter_mut() {
            let view_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: view_binding.clone(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: globals.clone(),
                    },
                ],
                label: Some("tilemap_view_bind_group"),
                layout: &tilemap_pipeline.view_layout,
            });

            commands.entity(entity).insert(TilemapViewBindGroup {
                value: view_bind_group,
            });

            let draw_tilemap = transparent_2d_draw_functions
                .read()
                .get_id::<DrawTilemapMaterial<M>>()
                .unwrap();

            for (entity, chunk_id, transform, tilemap_id) in standard_tilemap_meshes.iter() {
                if !visible_entities
                    .entities
                    .iter()
                    .any(|&entity| entity.index() == tilemap_id.0.index())
                {
                    continue;
                }

                let Ok(material_handle) = materials.get(tilemap_id.0) else {
                    continue;
                };
                let Some(material) = render_materials.get(material_handle) else {
                    continue;
                };

                if let Some(chunk) = chunk_storage.get(&UVec4::new(
                    chunk_id.0.x,
                    chunk_id.0.y,
                    chunk_id.0.z,
                    tilemap_id.0.index(),
                )) {
                    #[cfg(not(feature = "atlas"))]
                    if !texture_array_cache.contains(&chunk.texture) {
                        continue;
                    }

                    #[cfg(feature = "atlas")]
                    if gpu_images.get(chunk.texture.image_handle()).is_none() {
                        continue;
                    }

                    image_bind_groups
                        .values
                        .entry(chunk.texture.clone_weak())
                        .or_insert_with(|| {
                            #[cfg(not(feature = "atlas"))]
                            let gpu_image = texture_array_cache.get(&chunk.texture);
                            #[cfg(feature = "atlas")]
                            let gpu_image = gpu_images.get(chunk.texture.image_handle()).unwrap();
                            render_device.create_bind_group(&BindGroupDescriptor {
                                entries: &[
                                    BindGroupEntry {
                                        binding: 0,
                                        resource: BindingResource::TextureView(
                                            &gpu_image.texture_view,
                                        ),
                                    },
                                    BindGroupEntry {
                                        binding: 1,
                                        resource: BindingResource::Sampler(&gpu_image.sampler),
                                    },
                                ],
                                label: Some("sprite_material_bind_group"),
                                layout: &tilemap_pipeline.material_layout,
                            })
                        });

                    let key = TilemapPipelineKey {
                        msaa: msaa.samples(),
                        map_type: chunk.get_map_type(),
                        hdr: view.hdr,
                    };

                    let pipeline_id = material_pipelines.specialize(
                        &pipeline_cache,
                        &material_tilemap_pipeline,
                        MaterialTilemapKey {
                            tilemap_pipeline_key: key,
                            bind_group_data: material.key.clone(),
                        },
                    );
                    let z = if **y_sort {
                        transform.translation.z
                            + (1.0
                                - (transform.translation.y
                                    / (chunk.map_size.y as f32 * chunk.tile_size.y)))
                    } else {
                        transform.translation.z
                    };
                    transparent_phase.add(Transparent2d {
                        entity,
                        draw_function: draw_tilemap,
                        pipeline: pipeline_id,
                        sort_key: FloatOrd(z),
                        batch_range: None,
                    });
                }
            }
        }
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default, TypePath)]
#[uuid = "d6f8aeb8-510c-499a-9c0b-38551ae0b72a"]
pub struct StandardTilemapMaterial {}

impl MaterialTilemap for StandardTilemapMaterial {}
