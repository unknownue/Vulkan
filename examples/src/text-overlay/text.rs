
use ash::vk;
use memoffset::offset_of;

use rusttype::{Rect, VMetrics, HMetrics};

use std::ops::Range;
use std::collections::HashMap;

use vkbase::ci::buffer::BufferCI;
use vkbase::ci::memory::MemoryAI;
use vkbase::ci::image::{ImageCI, ImageViewCI, SamplerCI, ImageBarrierCI};
use vkbase::ci::pipeline::VertexInputSCI;
use vkbase::ci::command::{CommandPoolCI, CommandBufferAI};
use vkbase::ci::VkObjectBuildableCI;

use vkbase::command::VkCmdRecorder;
use vkbase::command::{IGraphics, CmdGraphicsApi};
use vkbase::command::{ITransfer, CmdTransferApi};

use vkbase::context::VkDevice;
use vkbase::utils::color::VkColor;

use vkbase::{vkuint, vkbytes, vkptr};
use vkbase::{VkResult, VkError};

const ASCII_RANGE: Range<u8> = (33..127_u8);
const VERTEX_PER_CHARACTER: usize = 6; // each character use 6 vertices to draw.
const TEXT_CAPABILITY_LENGTH: usize = 1024;
/// Control the font size of sampled glyph.
const FONT_SCALE: f32 = 48.0;
const DISPLAY_SCALE_FIX: f32 = 1.0 / 32.0; // magic number.
const IMAGE_PADDING: usize = 20;

type CharacterID = char;
type GlyphLayouts = HashMap<CharacterID, GlyphLayout>;

/// The vertices attributes for each character.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct CharacterVertex {
    pos   : [f32; 2],
    uv    : [f32; 2],
    color : [f32; 4],
}

#[derive(Debug, Clone)]
struct GlyphLayout {

    min_uv: [f32; 2],
    max_uv: [f32; 2],

    h_metrics: HMetrics,
    bounding_box: Rect<f32>,
}

pub struct GlyphImages {

    pub text_sampler: vk::Sampler,
    pub glyph_image: vk::Image,
    pub glyph_view : vk::ImageView,

    memory: vk::DeviceMemory,

    layouts: GlyphLayouts,
}

impl GlyphImages {

    pub fn from_font(device: &VkDevice, bytes: &[u8]) -> VkResult<GlyphImages> {

        let (layouts, image_bytes, image_dimension) =
            generate_ascii_glyphs_bytes(bytes, FONT_SCALE)?;
        let (glyph_image, memory) = allocate_image(device, image_bytes, image_dimension)?;

        // Just store alpha value in the image.
        let glyph_view = ImageViewCI::new(glyph_image, vk::ImageViewType::TYPE_2D, vk::Format::R8_UNORM)
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .build(device)?;
        let text_sampler = SamplerCI::new()
            .build(device)?;

        let result = GlyphImages { text_sampler, glyph_image, glyph_view, memory, layouts };
        Ok(result)
    }

    pub fn discard(&self, device: &VkDevice) {

        device.discard(self.text_sampler);
        device.discard(self.glyph_view);
        device.discard(self.glyph_image);
        device.discard(self.memory);
    }
}


pub struct TextPool {

    hidpi_factor: f32,
    // screen dimension.
    dimension: vk::Extent2D,
    texts: Vec<TextInfo>,
    texts_length: usize,

    data_ptr: vkptr,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

pub struct TextInfo {
    pub content: String,
    pub scale  : f32,
    pub color  : VkColor,
    pub location: vk::Offset2D,
}

impl TextPool {

    pub fn new(device: &VkDevice, dimension: vk::Extent2D, hidpi_factor: f32) -> VkResult<TextPool> {

        let pool_size = (::std::mem::size_of::<CharacterVertex>() * TEXT_CAPABILITY_LENGTH * VERTEX_PER_CHARACTER) as vkbytes;
        let (buffer, requirement) = BufferCI::new(pool_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build(device)?;

        let memory_type = device.get_memory_type(requirement.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        let memory = MemoryAI::new(requirement.size, memory_type)
            .build(device)?;
        device.bind_memory(buffer, memory, 0)?;
        // keep the memory mapping during the whole program running.
        let data_ptr = device.map_memory(memory, 0, vk::WHOLE_SIZE)?;

        let result = TextPool {
            texts: Vec::new(),
            texts_length: 0,
            hidpi_factor, buffer, memory, dimension, data_ptr,
        };
        Ok(result)
    }

    pub fn add_text(&mut self, mut text: TextInfo) -> VkResult<()> {

        if self.texts_length + text.content.len() <= TEXT_CAPABILITY_LENGTH {
            text.scale *= DISPLAY_SCALE_FIX * self.hidpi_factor;
            self.texts_length += text.content.len();
            self.texts.push(text);
            Ok(())
        } else {
            Err(VkError::custom("There is not enough room left for new text."))
        }
    }

    pub fn update_texts(&self, glyphs: &GlyphImages) -> VkResult<()> {

        // calculate vertices attributes of rendering texts.
        let mut char_vertices = Vec::with_capacity(self.texts_length * VERTEX_PER_CHARACTER);

        for text in self.texts.iter() {

            let mut origin_x = text.location.x as f32 / self.dimension.width as f32;
            let origin_y = text.location.y as f32 / self.dimension.height as f32;

            for ch in text.content.as_bytes() {

                let character_id = ch.clone() as char;
                let glyph_layout = glyphs.layouts.get(&character_id)
                    .ok_or(VkError::custom(format!("Find invalid character: {}({}).", character_id, character_id as u8)))?;

                let x_offset = (glyph_layout.bounding_box.min.x * text.scale) / self.dimension.width  as f32;
                let y_offset = (glyph_layout.bounding_box.min.y * text.scale) / self.dimension.height as f32;
                let glyph_width  = (glyph_layout.bounding_box.width()  * text.scale) / self.dimension.width  as f32;
                let glyph_height = (glyph_layout.bounding_box.height() * text.scale) / self.dimension.height as f32;

                // the x coordinate of top-left position(map to range [-1.0, 1.0]).
                let min_x = (origin_x + x_offset) * 2.0 - 1.0;
                // the y coordinate of top-left position.(map to range [-1.0, 1.0]).
                let min_y = (origin_y + y_offset) * 2.0 - 1.0;
                // the x coordinate of bottom-right position(map to range [-1.0, 1.0]).
                let max_x = (origin_x + glyph_width + x_offset) * 2.0 - 1.0;
                // the y coordinate of bottom-right position(map to range [-1.0, 1.0]).
                let max_y = (origin_y + glyph_height + y_offset) * 2.0 - 1.0;

                let top_left = CharacterVertex {
                    pos: [min_x, min_y],
                    uv: glyph_layout.min_uv,
                    color: text.color.into(),
                };
                let bottom_left = CharacterVertex {
                    pos: [min_x, max_y],
                    uv: [
                        glyph_layout.min_uv[0],
                        glyph_layout.max_uv[1],
                    ],
                    color: text.color.into(),
                };
                let bottom_right = CharacterVertex {
                    pos: [max_x, max_y],
                    uv: glyph_layout.max_uv,
                    color: text.color.into(),
                };
                let top_right = CharacterVertex {
                    pos: [max_x, min_y],
                    uv: [
                        glyph_layout.max_uv[0],
                        glyph_layout.min_uv[1],
                    ],
                    color: text.color.into(),
                };

                char_vertices.extend_from_slice(&[
                    top_left, bottom_left, bottom_right, // triangle 1
                    top_left, bottom_right, top_right,   // triangle 2
                ]);

                origin_x += (glyph_layout.h_metrics.advance_width * text.scale) / self.dimension.width as f32;
            }
        }

        // upload vertices attributes to memory.
        vkbase::utils::memory::copy_to_ptr(self.data_ptr, &char_vertices);

        Ok(())
    }

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>) {

        recorder.bind_vertex_buffers(0, &[self.buffer], &[0]);

        let vertex_count = (self.texts_length * VERTEX_PER_CHARACTER) as vkuint;
        recorder.draw(vertex_count, 1, 0, 0);
    }

    pub fn input_descriptions() -> VertexInputSCI {

        VertexInputSCI::new()
            .add_binding(vk::VertexInputBindingDescription {
                binding: 0,
                stride : ::std::mem::size_of::<CharacterVertex>() as _,
                input_rate: vk::VertexInputRate::VERTEX,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 0,
                binding : 0,
                format  : vk::Format::R32G32_SFLOAT,
                offset  : offset_of!(CharacterVertex, pos) as _,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 1,
                binding : 0,
                format  : vk::Format::R32G32_SFLOAT,
                offset  : offset_of!(CharacterVertex, uv) as _,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 2,
                binding : 0,
                format  : vk::Format::R32G32B32A32_SFLOAT,
                offset  : offset_of!(CharacterVertex, color) as _,
            })
    }

    pub fn discard(&self, device: &VkDevice) {

        device.unmap_memory(self.memory);
        device.discard(self.buffer);
        device.discard(self.memory);
    }
}

fn generate_ascii_glyphs_bytes(font_bytes: &[u8], font_scale: f32) -> VkResult<(GlyphLayouts, Vec<u8>, vk::Extent2D)> {

    use rusttype::{Font, Scale, PositionedGlyph, point};

    let font = Font::from_bytes(font_bytes)
        .map_err(|e| VkError::custom(e.to_string()))?;
    let ascii_bytes: Vec<u8> = ASCII_RANGE.collect();

    let ascii_characters = unsafe { String::from_utf8_unchecked(ascii_bytes.clone()) };

    let scale = Scale::uniform(font_scale);
    let v_metrics = font.v_metrics(scale);

    let glyphs_start_point = point(IMAGE_PADDING as f32, IMAGE_PADDING as f32 + v_metrics.ascent);
    let glyphs: Vec<PositionedGlyph> = font.layout(&ascii_characters, scale, glyphs_start_point)
        .collect();
    let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as usize;
    let glyphs_width = {
        let min_x = glyphs.first()
            .map(|g| g.pixel_bounding_box().unwrap().min.x)
            .unwrap();
        let max_x = glyphs.last()
            .map(|g| g.pixel_bounding_box().unwrap().max.x)
            .unwrap();
        (max_x - min_x) as usize
    };

    let image_width  = (2 * IMAGE_PADDING) + glyphs_width;
    let image_height = (2 * IMAGE_PADDING) + glyphs_height;
    let bytes_per_pixel = 1; // only store the alpha value.

    // fill image data with empty bytes.
    let mut image_bytes = vec![0_u8; image_width * image_height * bytes_per_pixel];

    let mut glyph_layouts = GlyphLayouts::new();

    // fill color to image data.
    for (glyph, character) in glyphs.iter().zip(ascii_bytes.into_iter()) {

        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            // Draw the glyph into the image per-pixel by using the draw closure.
            glyph.draw(|x, y, v| {

                let x = x + bounding_box.min.x as u32;
                let y = y + bounding_box.min.y as u32;
                let pos = (x + y * image_width as u32) as usize * bytes_per_pixel;

                image_bytes[pos] = (v * 255.0) as u8;
            });

            let min_uv = [
                bounding_box.min.x as f32 / image_width  as f32,
                bounding_box.min.y as f32 / image_height as f32,
            ];
            let max_uv = [
                bounding_box.max.x as f32 / image_width  as f32,
                bounding_box.max.y as f32 / image_height as f32,
            ];

            let glyph_unpositioned = glyph.unpositioned();
            let glyph_layout = GlyphLayout {
                min_uv, max_uv,
                h_metrics: glyph_unpositioned.h_metrics(),
                bounding_box: fix_bounding_box_positive(glyph_unpositioned.exact_bounding_box().unwrap(), &v_metrics),
            };
            glyph_layouts.insert(character as CharacterID, glyph_layout);
        }
    }

    // set the layout of space the same with 't'.
    let mut space_layout = glyph_layouts.get(&'t').unwrap().clone();
    space_layout.max_uv = space_layout.min_uv;
    glyph_layouts.insert(' ', space_layout);


    let dimension = vk::Extent2D {
        width : image_width  as vkuint,
        height: image_height as vkuint,
    };
    Ok((glyph_layouts, image_bytes, dimension))
}

fn allocate_image(device: &VkDevice, image_bytes: Vec<u8>, image_dimension: vk::Extent2D) -> VkResult<(vk::Image, vk::DeviceMemory)> {

    // create vk::Image and its memory.
    let (glyphs_image, image_reqs) = ImageCI::new_2d(vk::Format::R8_UNORM, image_dimension)
        .usages(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
        .build(device)?;
    let image_memory_type_index = device.get_memory_type(image_reqs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);
    let image_memory = MemoryAI::new(image_reqs.size, image_memory_type_index).build(device)?;
    device.bind_memory(glyphs_image, image_memory, 0)?;

    // create vk::Buffer and map image data to it.
    let estimate_buffer_size = (image_bytes.len() as vkbytes) * (::std::mem::size_of::<u8>() as vkbytes);
    let (staging_buffer, staging_reqs) = BufferCI::new(estimate_buffer_size)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC)
        .build(device)?;
    let staging_memory = MemoryAI::new(staging_reqs.size, device.get_memory_type(
        staging_reqs.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
    )).build(device)?;
    device.bind_memory(staging_buffer, staging_memory, 0)?;

    let data_ptr = device.map_memory(staging_memory, 0, vk::WHOLE_SIZE)?;
    vkbase::utils::memory::copy_to_ptr(data_ptr, &image_bytes);
    device.unmap_memory(staging_memory);

    // transfer image data from staging buffer to destination image.
    let command_pool = CommandPoolCI::new(device.logic.queues.transfer.family_index)
        .build(device)?;
    let copy_command = CommandBufferAI::new(command_pool, 1)
        .build(device)?
        .remove(0);

    let recorder: VkCmdRecorder<ITransfer> = VkCmdRecorder::new(&device.logic, copy_command);

    let copy_region = vk::BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length  : 0,
        buffer_image_height: 0,
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0, layer_count: 1,
        },
        image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        image_extent: vk::Extent3D { width: image_dimension.width, height: image_dimension.height, depth: 1 },
    };

    let image_range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level  : 0, level_count: 1,
        base_array_layer: 0, layer_count: 1,
    };
    let copy_dst_barrier = ImageBarrierCI::new(glyphs_image, image_range)
        .access_mask(vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE)
        .layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
    let shader_read_barrier = ImageBarrierCI::new(glyphs_image, image_range)
        .access_mask(vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ)
        .layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

    recorder.begin_record()?
        .image_pipeline_barrier(vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[copy_dst_barrier.value()])
        .copy_buf2img(staging_buffer, glyphs_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[copy_region])
        .image_pipeline_barrier(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &[shader_read_barrier.value()])
        .end_record()?;

    recorder.flush_copy_command(device.logic.queues.transfer.handle)?;

    // clean useless resources.
    device.discard(command_pool);
    device.discard(staging_buffer);
    device.discard(staging_memory);

    Ok((glyphs_image, image_memory))
}


// TODO: Fix and remove this magic function.
fn fix_bounding_box_positive(mut rect: Rect<f32>, v_metrics: &VMetrics) -> Rect<f32> {

    rect.min.y += v_metrics.ascent as f32;
    rect.max.y += v_metrics.ascent as f32;

    rect
}
