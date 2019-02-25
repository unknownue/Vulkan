
use ash::vk;
use memoffset::offset_of;

use vkbase::ci::buffer::BufferCI;
use vkbase::ci::memory::MemoryAI;
use vkbase::ci::pipeline::VertexInputSCI;
use vkbase::ci::VkObjectBuildableCI;

use vkbase::context::VkDevice;
use vkbase::command::{VkCmdRecorder, IGraphics};

use vkbase::{vkuint, vkbytes};
use vkbase::{VkResult, VkError};

pub const CHARACTER_COUNT: vkuint = 128;

type CharacterID = vkuint;
const TEXT_CAPABILITY_LENGTH: usize = 1024;

/// The vertex attributes for each character.
#[repr(C)]
#[derive(Debug, Clone)]
struct CharacterVertex {
    id    : CharacterID,
    pos   : [f32; 2],
    uv    : [f32; 2],
    color : [f32; 4],
}

pub struct CharacterImage {
    pub image: vk::Image,
    pub view : vk::ImageView,
    pub metrics: CharacterMetrics,
}

pub struct CharacterMetrics {

}

pub struct GlyphImages {
    pub text_sampler: vk::Sampler,
    pub images: [CharacterImage; CHARACTER_COUNT as usize],
    pub memory: vk::DeviceMemory,
}

impl GlyphImages {

    pub fn from_font(bytes: &[u8]) -> GlyphImages {
        unimplemented!()
    }

    pub fn discard(&self, device: &VkDevice) {
        device.discard(self.text_sampler);

        for glyph in self.images.iter() {
            device.discard(glyph.view);
            device.discard(glyph.image);
        }

        device.discard(self.memory);
    }
}


pub struct TextPool {

    texts: Vec<String>,
    texts_length: usize,

    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl TextPool {

    pub fn new(device: &VkDevice) -> VkResult<TextPool> {

        let pool_size = (::std::mem::size_of::<CharacterVertex>() * TEXT_CAPABILITY_LENGTH) as vkbytes;
        let (buffer, requirement) = BufferCI::new(pool_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build(device)?;

        let memory_type = device.get_memory_type(requirement.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        let memory = MemoryAI::new(requirement.size, memory_type)
            .build(device)?;

        let result = TextPool {
            texts: Vec::new(),
            texts_length: 0,
            buffer, memory,
        };
        Ok(result)
    }

    pub fn add_text(&mut self, text: String) -> VkResult<()> {

        if self.texts_length + text.len() <= TEXT_CAPABILITY_LENGTH {
            self.texts_length += text.len();
            self.texts.push(text);
            Ok(())
        } else {
            Err(VkError::custom("There is not enough room left for new text."))
        }
    }

    pub fn update_texts(&self, device: &VkDevice, glyphs: &GlyphImages) {
        unimplemented!()
    }

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>) {

        unimplemented!()
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
                format  : vk::Format::R32_UINT,
                offset  : offset_of!(CharacterVertex, id) as _,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 0,
                binding : 1,
                format  : vk::Format::R32G32_SFLOAT,
                offset  : offset_of!(CharacterVertex, pos) as _,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 0,
                binding : 2,
                format  : vk::Format::R32G32_SFLOAT,
                offset  : offset_of!(CharacterVertex, uv) as _,
            })
            .add_attribute(vk::VertexInputAttributeDescription {
                location: 0,
                binding : 3,
                format  : vk::Format::R32G32B32A32_SFLOAT,
                offset  : offset_of!(CharacterVertex, color) as _,
            })
    }

    pub fn discard(&self, device: &VkDevice) {
        device.discard(self.buffer);
        device.discard(self.memory);
    }
}
