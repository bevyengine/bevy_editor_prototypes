use bevy::{
    asset::AssetId,
    prelude::{Deref, DerefMut, Image, Res, Resource, World},
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, Extent3d, ImageCopyBuffer, ImageDataLayout,
            Maintain, MapMode,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
    },
    scene::Scene,
    utils::HashMap,
};
use crossbeam_channel::{Receiver, Sender};

#[derive(Clone, Deref)]
pub struct PreviewImageCopy(Buffer);

impl PreviewImageCopy {
    pub fn new(width: u32, height: u32, render_device: &RenderDevice) -> Self {
        let padding_bytes_per_row = RenderDevice::align_copy_bytes_per_row(width as usize * 4);
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: padding_bytes_per_row as u64 * height as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self(buffer)
    }
}

#[derive(ExtractResource, Resource, Default, Clone, Deref, DerefMut)]
pub struct PreviewImageCopies(HashMap<AssetId<Image>, PreviewImageCopy>);

#[derive(Resource, Default, Clone, Deref, DerefMut)]
pub struct TransferredPreviewImages(Vec<(AssetId<Scene>, AssetId<Image>)>);

#[derive(Resource, Deref)]
pub struct MainWorldPreviewImageReceiver(pub Receiver<(AssetId<Image>, Vec<u8>)>);

#[derive(Resource, Deref)]
pub struct RenderWorldPreviewImageSender(pub Sender<(AssetId<Image>, Vec<u8>)>);

#[derive(RenderLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PreviewTextureToBufferLabel;

pub struct PreviewTextureToBufferNode;

impl Node for PreviewTextureToBufferNode {
    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let image_copies = world.resource::<PreviewImageCopies>();
        let images = world.resource::<RenderAssets<GpuImage>>();

        for (id, buffer) in &**image_copies {
            let src = images.get(*id).unwrap();
            let mut encoder = render_context
                .render_device()
                .create_command_encoder(&Default::default());

            let block_dimension = src.texture_format.block_dimensions();
            let block_size = src.texture_format.block_copy_size(None).unwrap();

            let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
                (src.size.x as usize / block_dimension.0 as usize) * block_size as usize,
            );

            encoder.copy_texture_to_buffer(
                src.texture.as_image_copy(),
                ImageCopyBuffer {
                    buffer: &buffer,
                    layout: ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some((padded_bytes_per_row as u32).into()),
                        rows_per_image: None,
                    },
                },
                Extent3d {
                    width: src.size.x,
                    height: src.size.y,
                    depth_or_array_layers: 1,
                },
            );

            world.resource::<RenderQueue>().submit([encoder.finish()]);
        }

        Ok(())
    }
}

pub fn receive_image_from_buffer(
    image_copies: Res<PreviewImageCopies>,
    render_device: Res<RenderDevice>,
    sender: Res<RenderWorldPreviewImageSender>,
) {
    for (id, copy) in &**image_copies {
        let buffer_slice = copy.slice(..);

        let (s, r) = crossbeam_channel::bounded(1);
        buffer_slice.map_async(MapMode::Read, move |r| match r {
            Ok(r) => s.send(r).expect("Failed to send map update."),
            Err(err) => panic!("Failed to map preview image buffer {:?}", err),
        });

        render_device.poll(Maintain::wait()).panic_on_timeout();
        r.recv().expect("Failed to receive the map_async message");

        let _ = sender.send((*id, buffer_slice.get_mapped_range().to_vec()));
        copy.unmap();
    }
}
