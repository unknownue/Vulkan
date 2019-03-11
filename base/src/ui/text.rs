
use ash::vk;
use memoffset::offset_of;

use rusttype::{Rect, VMetrics, HMetrics};

use std::ops::Range;
use std::collections::HashMap;
use std::iter::Iterator;

use crate::ci::buffer::BufferCI;
use crate::ci::memory::MemoryAI;
use crate::ci::image::{ImageCI, ImageViewCI, SamplerCI, ImageBarrierCI};
use crate::ci::vma::{VmaBuffer, VmaImage, VmaAllocationCI};
use crate::ci::pipeline::VertexInputSCI;
use crate::ci::VkObjectBuildableCI;

use crate::context::VkDevice;
use crate::command::{VkCmdRecorder, IGraphics, CmdGraphicsApi, CmdTransferApi};

use crate::utils::color::VkColor;
use crate::{vkuint, vkbytes, vkptr};
use crate::{VkResult, VkError, VkErrorKind};


/// each character use 6 vertices to draw.
const VERTEX_PER_CHARACTER: usize = 6;
/// the maximum sentence count that the buffer can contain.
const MAXIMUM_SENTENCE_COUNT: usize = 10;
/// the maximum character count that a sentence may contain.
const MAXIMUM_SENTENCE_TEXT_COUNT: usize = 100;
/// Control the font size of sampled glyph.
const FONT_SCALE: f32 = 48.0;
/// A magic number.
const DISPLAY_SCALE_FIX: f32 = 1.0 / 768.0;
/// The padding attach to sampled glyph image.
const IMAGE_PADDING: usize = 20;

pub type TextID = usize;
type CharacterID = char;
type GlyphLayouts = HashMap<CharacterID, GlyphLayout>;

/// The vertices attributes for each character.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
    pub glyph_image: VmaImage,
    pub glyph_view : vk::ImageView,

    layouts: GlyphLayouts,
}

impl GlyphImages {

    pub fn from_font(device: &mut VkDevice, bytes: &[u8]) -> VkResult<GlyphImages> {

        let (layouts, image_bytes, image_dimension) =
            generate_ascii_glyphs_bytes(bytes, FONT_SCALE)?;
        let glyph_image = allocate_glyph_image(device, image_bytes, image_dimension)?;

        // Just store alpha value in the image.
        let glyph_view = ImageViewCI::new(glyph_image.handle, vk::ImageViewType::TYPE_2D, vk::Format::R8_UNORM)
            .sub_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count   : 1,
                base_array_layer: 0,
                layer_count     : 1,
            }).build(device)?;

        let text_sampler = SamplerCI::new()
            .build(device)?;

        let result = GlyphImages { text_sampler, glyph_image, glyph_view, layouts };
        Ok(result)
    }

    pub fn discard(self, device: &mut VkDevice) -> VkResult<()> {

        device.discard(self.text_sampler);
        device.discard(self.glyph_view);
        device.vma_discard(self.glyph_image)
    }
}

struct TextAttrStorage {
    /// the starting pointer of the memory of text attributes.
    data_ptr: vkptr,
    /// the buffer which store the text attributes.
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl TextAttrStorage {

    fn new(device: &VkDevice) -> VkResult<TextAttrStorage> {

        let pool_size = (::std::mem::size_of::<CharacterVertex>() * MAXIMUM_SENTENCE_COUNT * MAXIMUM_SENTENCE_TEXT_COUNT * VERTEX_PER_CHARACTER) as vkbytes;
        let (buffer, requirement) = BufferCI::new(pool_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build(device)?;

        let memory_type = device.get_memory_type(requirement.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        let memory = MemoryAI::new(requirement.size, memory_type)
            .build(device)?;
        device.bind_memory(buffer, memory, 0)?;
        // keep the memory mapping during the whole program running.
        let data_ptr = device.map_memory(memory, 0, vk::WHOLE_SIZE)?;

        let result = TextAttrStorage { data_ptr, buffer, memory };
        Ok(result)
    }

    fn discard(self, device: &VkDevice) {

        device.unmap_memory(self.memory);
        device.discard(self.buffer);
        device.discard(self.memory);
    }
}


pub struct TextPool {

    /// screen dimension of current window.
    dimension: vk::Extent2D,
    // the aspect ratio of current screen dimension.
    aspect_ratio: f32,

    /// all the texts to be rendered.
    texts: Vec<TextInfo>,
    /// `attributes` contains the resource for rendering texts.
    attributes: TextAttrStorage,
    /// `glyph_layouts` records the layout information to generate text attributes.
    glyphs: GlyphImages,
}

pub struct TextInfo {
    /// `content` is the content of text to render.
    pub content: String,
    /// `scale` defines the font size of this text.
    pub scale  : f32,
    /// `align` the align method for this text.
    pub align  : TextHAlign,
    /// `color` is color value of this text.
    pub color  : VkColor,
    /// `location` is the starting position of the first character.
    pub location: vk::Offset2D,

    pub r#type: TextType,
}

pub enum TextType {
    /// Render static text to screen. The text can not change after first set.
    Static,
    /// Render text that is dynamically changed in runtime.
    ///
    /// `capacity` is the maximum length of the text to rendering.
    ///
    /// Use `change_text` method to set the content of text in runtime.
    Dynamic { capacity: usize },
}

pub struct TextIter<'a> {
    content: &'a [u8],
    current: usize,
    len: usize,
    capacity: usize,
}

impl<'a> Iterator for TextIter<'a> {
    type Item = Option<char>;

    fn next(&mut self) -> Option<Self::Item> {

        let result = if self.current < self.len {
            Some(self.content[self.current] as char)
        } else {
            if self.current < self.capacity {
                None
            } else {
                return None
            }
        };

        self.current += 1;
        Some(result)
    }
}

impl TextInfo {

    fn iter(&self) -> TextIter {

        let char_bytes = self.content.as_bytes();
        let bytes_length = char_bytes.len();

        let mut iter = TextIter {
            content: char_bytes,
            current: 0,
            len: bytes_length,
            capacity: 0,
        };

        iter.capacity = match self.r#type {
            | TextType::Static => bytes_length,
            | TextType::Dynamic { capacity } => capacity,
        };
        iter
    }
}

/// The horizontal align of a specific text.
pub enum TextHAlign {
    Left,
    Center,
    Right,
}

impl TextPool {

    pub fn new(device: &mut VkDevice, dimension: vk::Extent2D) -> VkResult<TextPool> {

        let attributes = TextAttrStorage::new(device)?;

        let font_bytes = include_bytes!("../../../assets/fonts/Roboto-Regular.ttf");
        let glyphs = GlyphImages::from_font(device, font_bytes)?;

        let result = TextPool {
            texts: Vec::new(),
            aspect_ratio: dimension.width as f32 / dimension.height as f32,
            attributes, glyphs, dimension,
        };
        Ok(result)
    }

    pub fn add_text(&mut self, mut text: TextInfo) -> VkResult<TextID> {

        if self.texts.len() < MAXIMUM_SENTENCE_COUNT {
            if text.content.len() <= MAXIMUM_SENTENCE_TEXT_COUNT {

                text.scale *= DISPLAY_SCALE_FIX / FONT_SCALE;

                let new_text_id = self.texts.len();
                self.texts.push(text);
                // update the text that is newly added.
                self.update_texts(new_text_id);

                Ok(new_text_id)
            } else {
                Err(VkError::custom(format!("Each sentence can't contain more that {} character.", MAXIMUM_SENTENCE_TEXT_COUNT)))
            }
        } else {
            Err(VkError::custom(format!("The text pool can't contain more than {} sentence.", MAXIMUM_SENTENCE_COUNT)))
        }
    }

    pub fn change_text(&mut self, content: String, update_text: TextID) {

        self.texts[update_text].content = content;
        self.update_texts(update_text);
    }

    fn update_texts(&self, update_text: TextID) {

        // calculate vertices attributes of rendering texts.
        let mut char_vertices = Vec::with_capacity(self.texts.len() * MAXIMUM_SENTENCE_TEXT_COUNT * VERTEX_PER_CHARACTER);

        let text = &self.texts[update_text];

        let mut origin_x = text.location.x as f32 / self.dimension.width as f32;
        let origin_y = text.location.y as f32 / self.dimension.height as f32;

        for ch in text.iter() {

            // use ' '(space) character instead if all the characters of current text has been rendered, but not yet reached its capacity.
            let character_id = ch.unwrap_or(' ');

            let glyph_layout = self.glyphs.layouts.get(&character_id)
                .expect(&format!("Find invalid character: {}({}).", character_id, character_id as u8));

            let x_offset     = glyph_layout.bounding_box.min.x    * text.scale;
            let y_offset     = glyph_layout.bounding_box.min.y    * text.scale * self.aspect_ratio;
            let glyph_width  = glyph_layout.bounding_box.width()  * text.scale;
            let glyph_height = glyph_layout.bounding_box.height() * text.scale * self.aspect_ratio;

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

            origin_x += glyph_layout.h_metrics.advance_width * text.scale;
        }

        // adjust the position of each vertices to make text alignment.
        match text.align {
            | TextHAlign::Left => {
                // currently the text is left align.
            },
            | TextHAlign::Center => {
                // move the text the center position.
                let text_half_length = origin_x - text.location.x as f32 / self.dimension.width as f32;
                for char_vertex in char_vertices.iter_mut() {
                    char_vertex.pos[0] -= text_half_length;
                }
            },
            | TextHAlign::Right => {
                // make text align to right.
                let text_half_length = origin_x - text.location.x as f32 / self.dimension.width as f32;
                let text_length = text_half_length * 2.0;
                for char_vertex in char_vertices.iter_mut() {
                    char_vertex.pos[0] -= text_length; // pos[0] is the x coordinate.
                }
            },
        }

        // upload vertices attributes to memory.
        unsafe {
            let target_ptr = (self.attributes.data_ptr as vkptr<CharacterVertex>)
                .offset((MAXIMUM_SENTENCE_TEXT_COUNT * VERTEX_PER_CHARACTER * update_text) as isize);
            target_ptr.copy_from(char_vertices.as_ptr(), char_vertices.len());
        }
    }

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>) {

        recorder.bind_vertex_buffers(0, &[self.attributes.buffer], &[0]);

        let mut first_vertex = 0;
        for text in self.texts.iter() {

            let character_count = match text.r#type {
                | TextType::Static => text.content.len(),
                | TextType::Dynamic { capacity } => capacity,
            };
            let render_vertex_count = (character_count * VERTEX_PER_CHARACTER) as vkuint;
            recorder.draw(render_vertex_count, 1, first_vertex, 0);
            first_vertex += (MAXIMUM_SENTENCE_TEXT_COUNT * VERTEX_PER_CHARACTER) as vkuint;
        }
    }

    pub fn swapchain_reload(&mut self) {

        for i in 0..self.texts.len() {
            self.update_texts(i);
        }
    }

    pub fn glyphs_ref(&self) -> &GlyphImages {
        &self.glyphs
    }

    pub fn discard_by(self, device: &mut VkDevice) -> VkResult<()> {

        self.attributes.discard(device);
        self.glyphs.discard(device)
    }
}

fn generate_ascii_glyphs_bytes(font_bytes: &[u8], font_scale: f32) -> VkResult<(GlyphLayouts, Vec<u8>, vk::Extent2D)> {

    use rusttype::{Font, Scale, PositionedGlyph, point};

    /// the ascii character range that render to sampled glyph.
    const ASCII_RANGE: Range<u8> = (33..127_u8);

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

    // set the layout of space the same with 't', since space does not have a bounding box.
    let mut space_layout = glyph_layouts.get(&'t').unwrap().clone();
    // set the same uv for min and max position, so that nothing will be render for space.
    space_layout.max_uv = space_layout.min_uv;
    glyph_layouts.insert(' ', space_layout);


    let dimension = vk::Extent2D {
        width : image_width  as vkuint,
        height: image_height as vkuint,
    };
    Ok((glyph_layouts, image_bytes, dimension))
}

fn allocate_glyph_image(device: &mut VkDevice, image_bytes: Vec<u8>, image_dimension: vk::Extent2D) -> VkResult<VmaImage> {

    // create vk::Image to store glyphs data.
    let glyphs_image = {

        let glyphs_image_ci = ImageCI::new_2d(vk::Format::R8_UNORM, image_dimension)
            .usages(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::GpuOnly, vk::MemoryPropertyFlags::DEVICE_LOCAL);
        let image_allocation = device.vma.create_image(&glyphs_image_ci.value(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;
        VmaImage::from(image_allocation)
    };

    // create staging buffer and map image data to it.
    let staging_buffer = {

        let estimate_buffer_size = (image_bytes.len() as vkbytes) * (::std::mem::size_of::<u8>() as vkbytes);
        let staging_ci = BufferCI::new(estimate_buffer_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC);
        let allocation_ci = VmaAllocationCI::new(vma::MemoryUsage::CpuToGpu, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        let staging_allocation = device.vma.create_buffer(&staging_ci.value(), allocation_ci.as_ref())
            .map_err(VkErrorKind::Vma)?;

        let data_ptr = device.vma.map_memory(&staging_allocation.1)
            .map_err(VkErrorKind::Vma)? as vkptr;
        crate::utils::memory::copy_to_ptr(data_ptr, &image_bytes);
        device.vma.unmap_memory(&staging_allocation.1)
            .map_err(VkErrorKind::Vma)?;

        VmaBuffer::from(staging_allocation)
    };

    // transfer image data from staging buffer to destination image.
    let recorder = device.get_transfer_recorder();

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
    let copy_dst_barrier = ImageBarrierCI::new(glyphs_image.handle, image_range)
        .access_mask(vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE)
        .layout(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
    let shader_read_barrier = ImageBarrierCI::new(glyphs_image.handle, image_range)
        .access_mask(vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ)
        .layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

    recorder.begin_record()?
        .image_pipeline_barrier(vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[copy_dst_barrier.value()])
        .copy_buf2img(staging_buffer.handle, glyphs_image.handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[copy_region])
        .image_pipeline_barrier(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &[shader_read_barrier.value()])
        .end_record()?;

    device.flush_transfer(recorder)?;

    // clean useless resources.
    device.vma_discard(staging_buffer)?;

    Ok(glyphs_image)
}


// TODO: Fix and remove this magic function.
fn fix_bounding_box_positive(mut rect: Rect<f32>, v_metrics: &VMetrics) -> Rect<f32> {

    rect.min.y += v_metrics.ascent as f32;
    rect.max.y += v_metrics.ascent as f32;

    rect
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
