use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TileComponent)]
pub fn tile_component(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let target = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        unsafe impl<#target: Sized + Send + Sync + 'static> TileComponent for #target {
            #[inline]
            fn insert_tile_into_chunk<const N: usize>(
                self,
                map_id: Entity,
                mut chunk: EntityWorldMut<'_>,
                chunk_size: usize,
                tile_c: [i32; N],
                tile_i: usize,
            ) -> Option<Self> {
                 match chunk.get_mut::<ChunkData<Self>>() {
                    Some(data) => data,
                    None => {
                        chunk
                            .get_mut::<ChunkTypes>()
                            .unwrap()
                            .0
                            .insert(TypeId::of::<Self>());
                        let chunk = chunk.insert(ChunkData::<Self>::new(
                            chunk_size.pow(N.try_into().unwrap()),
                        ));
                        chunk.get_mut::<ChunkData<Self>>().unwrap()
                    }
                }.insert(tile_i, self)
            }

            #[inline]
            fn take_tile_from_chunk(chunk: &mut EntityWorldMut<'_>, tile_i: usize) -> Option<Self> {
                let location = chunk.get_mut::<ChunkData<Self>>();
                let mut binding = location?;
                let removed = binding.take(tile_i);
                if binding.count == 0 {
                    chunk
                        .get_mut::<ChunkTypes>()
                        .unwrap()
                        .0
                        .remove(&TypeId::of::<Self>());
                    chunk.remove::<ChunkData<Self>>();
                }
                removed
            }

            #[inline]
            fn insert_tile_batch_into_chunk<const N: usize>(
                tiles: impl Iterator<Item = Self>,
                map_id: Entity,
                mut chunk: EntityWorldMut<'_>,
                chunk_size: usize,
                tile_info: impl Iterator<Item = ([i32; N], usize)>,
            ) {
                let mut location = match chunk.get_mut::<ChunkData<Self>>() {
                    Some(data) => data,
                    None => {
                        chunk
                            .get_mut::<ChunkTypes>()
                            .unwrap()
                            .0
                            .insert(TypeId::of::<Self>());
                        let chunk = chunk.insert(ChunkData::<Self>::new(
                            chunk_size.pow(N.try_into().unwrap()),
                        ));
                        chunk.get_mut::<ChunkData<Self>>().unwrap()
                    }
                };
                for ((_, tile_i), tile) in tile_info.zip(tiles) {
                    location.insert(tile_i, tile);
                }
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

// /// # Safety:
// /// Probably safe.
// /// MAKE THIS NOT A DEFAULT IMPL
// 