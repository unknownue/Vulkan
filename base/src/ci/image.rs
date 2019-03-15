
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::{VkDevice, VkObjectDiscardable, VkObjectBindable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::{vkbytes, vkuint, vkfloat};

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::ImageCreateInfo.
#[derive(Debug, Clone)]
pub struct ImageCI {

    inner: vk::ImageCreateInfo,
    queue_families: Option<Vec<vkuint>>,
}

impl VulkanCI<vk::ImageCreateInfo> for ImageCI {

    fn default_ci() -> vk::ImageCreateInfo {

        vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::UNDEFINED,
            extent: Default::default(),
            mip_levels  : 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling : vk::ImageTiling::OPTIMAL,
            usage  : vk::ImageUsageFlags::empty(),
            sharing_mode  : vk::SharingMode::EXCLUSIVE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            queue_family_index_count: 0,
            p_queue_family_indices  : ptr::null(),
        }
    }
}

impl AsRef<vk::ImageCreateInfo> for ImageCI {

    fn as_ref(&self) -> &vk::ImageCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for ImageCI {
    type ObjectType = (vk::Image, vk::MemoryRequirements);

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let image = unsafe {
            device.logic.handle.create_image(self.as_ref(), None)
                .map_err(|_| VkError::create("Image"))?
        };

        let requirement = unsafe {
            device.logic.handle.get_image_memory_requirements(image)
        };

        Ok((image, requirement))
    }
}

impl ImageCI {

    pub fn new(r#type: vk::ImageType, format: vk::Format, dimension: vk::Extent3D) -> ImageCI {

        ImageCI {
            inner: vk::ImageCreateInfo {
                image_type: r#type,
                format,
                extent: dimension,
                ..ImageCI::default_ci()
            },
            queue_families: None,
        }
    }

    pub fn new_2d(format: vk::Format, dimension: vk::Extent2D) -> ImageCI {

        let extent = vk::Extent3D {
            width : dimension.width,
            height: dimension.height,
            depth : 1,
        };

        ImageCI::new(vk::ImageType::TYPE_2D, format, extent)
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::ImageCreateFlags) -> ImageCI {
        self.inner.flags = flags; self
    }

    #[inline(always)]
    pub fn usages(mut self, flags: vk::ImageUsageFlags) -> ImageCI {
        self.inner.usage = flags; self
    }

    #[inline(always)]
    pub fn tiling(mut self, tiling: vk::ImageTiling) -> ImageCI {
        self.inner.tiling = tiling; self
    }

    #[inline(always)]
    pub fn samples(mut self, count: vk::SampleCountFlags) -> ImageCI {
        self.inner.samples = count; self
    }

    #[inline(always)]
    pub fn mip_levels(mut self, level: vkuint) -> ImageCI {
        self.inner.mip_levels = level; self
    }

    #[inline(always)]
    pub fn array_layers(mut self, layers: vkuint) -> ImageCI {
        self.inner.array_layers = layers; self
    }

    #[inline(always)]
    pub fn initial_layout(mut self, layout: vk::ImageLayout) -> ImageCI {
        self.inner.initial_layout = layout; self
    }

    #[inline(always)]
    pub fn sharing_queues(mut self, mode: vk::SharingMode, families_indices: Vec<vkuint>) -> ImageCI {

        self.inner.queue_family_index_count = families_indices.len() as _;
        self.inner.p_queue_family_indices   = families_indices.as_ptr();

        self.queue_families = Some(families_indices);
        self.inner.sharing_mode = mode; self
    }
}

impl VkObjectDiscardable for vk::Image {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_image(self, None);
        }
    }
}

impl VkObjectBindable for vk::Image {

    fn bind(self, device: &VkDevice, memory: vk::DeviceMemory, offset: vkbytes) -> VkResult<()> {
        unsafe {
            device.logic.handle.bind_image_memory(self, memory, offset)
                .map_err(|_| VkError::device("Binding Image Memory"))
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::ImageViewCreateInfo.
#[derive(Debug, Clone)]
pub struct ImageViewCI {
    inner: vk::ImageViewCreateInfo,
}

impl VulkanCI<vk::ImageViewCreateInfo> for ImageViewCI {

    fn default_ci() -> vk::ImageViewCreateInfo {

        vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::ImageViewCreateFlags::empty(),
            image : vk::Image::null(),
            view_type: vk::ImageViewType::TYPE_2D,
            format: vk::Format::UNDEFINED,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask      : vk::ImageAspectFlags::COLOR,
                base_mip_level   : 0,
                level_count      : 1,
                base_array_layer : 0,
                layer_count      : 1,
            },
        }
    }
}

impl AsRef<vk::ImageViewCreateInfo> for ImageViewCI {

    fn as_ref(&self) -> &vk::ImageViewCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for ImageViewCI {
    type ObjectType = vk::ImageView;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let view = unsafe {
            device.logic.handle.create_image_view(self.as_ref(), None)
                .map_err(|_| VkError::create("Image View"))?
        };
        Ok(view)
    }
}

impl ImageViewCI {

    pub fn new(image: vk::Image, r#type: vk::ImageViewType, format: vk::Format) -> ImageViewCI {

        ImageViewCI {
            inner: vk::ImageViewCreateInfo {
                image, format,
                view_type: r#type,
                ..ImageViewCI::default_ci()
            },
        }
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::ImageViewCreateFlags) -> ImageViewCI {
        self.inner.flags = flags; self
    }

    #[inline(always)]
    pub fn components(mut self, components: vk::ComponentMapping) -> ImageViewCI {
        self.inner.components = components;; self
    }

    #[inline(always)]
    pub fn sub_range(mut self, range: vk::ImageSubresourceRange) -> ImageViewCI {
        self.inner.subresource_range = range; self
    }
}

impl VkObjectDiscardable for vk::ImageView {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_image_view(self, None)
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::ImageMemoryBarrier.
#[derive(Debug, Clone)]
pub struct ImageBarrierCI {
    inner: vk::ImageMemoryBarrier,
}

impl VulkanCI<vk::ImageMemoryBarrier> for ImageBarrierCI {

    fn default_ci() -> vk::ImageMemoryBarrier {

        vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::UNDEFINED,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: vk::Image::null(),
            subresource_range: Default::default(),
        }
    }
}

impl AsRef<vk::ImageMemoryBarrier> for ImageBarrierCI {

    fn as_ref(&self) -> &vk::ImageMemoryBarrier {
        &self.inner
    }
}

impl ImageBarrierCI {

    pub fn new(image: vk::Image, subrange: vk::ImageSubresourceRange) -> ImageBarrierCI {

        ImageBarrierCI {
            inner: vk::ImageMemoryBarrier {
                image,
                subresource_range: subrange,
                ..ImageBarrierCI::default_ci()
            },
        }
    }

    #[inline(always)]
    pub fn access_mask(mut self, from: vk::AccessFlags, to: vk::AccessFlags) -> Self {
        self.inner.src_access_mask = from;
        self.inner.dst_access_mask = to; self
    }

    #[inline(always)]
    pub fn layout(mut self, from: vk::ImageLayout, to: vk::ImageLayout) -> Self {
        self.inner.old_layout = from;
        self.inner.new_layout = to; self
    }

    #[inline(always)]
    pub fn queue_family_index(mut self, from: vkuint, to: vkuint) -> Self {
        self.inner.src_queue_family_index = from;
        self.inner.dst_queue_family_index = to; self
    }
}

impl From<ImageBarrierCI> for vk::ImageMemoryBarrier {

    fn from(v: ImageBarrierCI) -> vk::ImageMemoryBarrier {
        v.inner
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for vk::SamplerCreateInfo.
pub struct SamplerCI {
    inner: vk::SamplerCreateInfo,
}

impl VulkanCI<vk::SamplerCreateInfo> for SamplerCI {

    fn default_ci() -> vk::SamplerCreateInfo {

        vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags : vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: vk::FALSE,
            max_anisotropy   : 1.0,
            compare_enable: vk::FALSE,
            compare_op    : vk::CompareOp::ALWAYS,
            mip_lod_bias: 0.0,
            min_lod     : 0.0,
            max_lod     : 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        }
    }
}

impl AsRef<vk::SamplerCreateInfo> for SamplerCI {

    fn as_ref(&self) -> &vk::SamplerCreateInfo {
        &self.inner
    }
}

impl SamplerCI {

    #[inline(always)]
    pub fn new() -> SamplerCI {
        SamplerCI {
            inner: SamplerCI::default_ci(),
        }
    }

    pub fn build(&self, device: &VkDevice) -> VkResult<vk::Sampler> {

        let sampler = unsafe {
            device.logic.handle.create_sampler(self.as_ref(), None)
                .map_err(|_| VkError::create("Sampler"))?
        };
        Ok(sampler)
    }

    /// `mag` specifies the magnification filter to apply to lookups.
    ///
    /// `min` specifies the minification filter to apply to lookups.
    #[inline(always)]
    pub fn filter(mut self, mag: vk::Filter, min: vk::Filter) -> SamplerCI {
        self.inner.mag_filter = mag;
        self.inner.min_filter = min; self
    }

    /// `mode` specifies the mipmap filter to apply to lookups.
    #[inline(always)]
    pub fn mipmap(mut self, mode: vk::SamplerMipmapMode) -> SamplerCI {
        self.inner.mipmap_mode = mode; self
    }

    /// `u`, `v` and `w` specifies the addressing mode for outside [0..1] range for U, V, W coordinate.
    #[inline(always)]
    pub fn address(mut self, u: vk::SamplerAddressMode, v: vk::SamplerAddressMode, w: vk::SamplerAddressMode) -> SamplerCI {
        self.inner.address_mode_u = u;
        self.inner.address_mode_v = v;
        self.inner.address_mode_w = w; self
    }

    /// `mip_bias` is the bias to be added to mipmap LOD (level-of-detail) calculation and bias provided by image sampling functions in SPIR-V.
    ///
    /// `min` used to clamp the minimum computed LOD value, as described in the Level-of-Detail Operation section.
    ///
    /// `max` used to clamp the maximum computed LOD value, as described in the Level-of-Detail Operation section.
    #[inline(always)]
    pub fn lod(mut self, mip_bias: vkfloat, min: vkfloat, max: vkfloat) -> SamplerCI {
        self.inner.mip_lod_bias = mip_bias;
        self.inner.min_lod = min;
        self.inner.max_lod = max; self
    }

    /// This function needs to enable an physical feature named 'sampler_anisotropy'.
    ///
    /// `max` is the anisotropy value clamp used by the sampler.
    ///
    /// If `max` is None, anisotropy will be disabled.
    #[inline(always)]
    pub fn anisotropy(mut self, max: Option<vkfloat>) -> SamplerCI {

        if let Some(max) = max {
            self.inner.anisotropy_enable = vk::TRUE;
            self.inner.max_anisotropy = max;
        } else {
            self.inner.anisotropy_enable = vk::FALSE;
        }

        self
    }

    /// `op` specifies the comparison function to apply to fetched data before filtering
    /// as described in the Depth Compare Operation section.
    ///
    /// Set `op` to some value to enable comparison.
    ///
    /// If `op` is None, the compare function will be disabled.
    #[inline(always)]
    pub fn compare_op(mut self, op: Option<vk::CompareOp>) -> SamplerCI {

        if let Some(op) = op  {
            self.inner.compare_enable = vk::TRUE;
            self.inner.compare_op = op;
        } else {
            self.inner.compare_enable = vk::FALSE;
        }

        self
    }

    /// `border_color` specifies the predefined border color to use.
    #[inline(always)]
    pub fn border_color(mut self, color: vk::BorderColor) -> SamplerCI {
        self.inner.border_color = color; self
    }

    /// `unnormalize_coordinates_enable` controls whether to use unnormalized or normalized texel coordinates to address texels of the image.
    ///
    /// When set to true, the range of the image coordinates used to lookup the texel is in the range of zero
    /// to the image dimensions for x, y and z.
    ///
    /// When set to false, the range of image coordinates is zero to one.
    #[inline(always)]
    pub fn unnormalize_coordinates_enable(mut self, enable: bool) -> SamplerCI {
        self.inner.unnormalized_coordinates = if enable { vk::TRUE } else { vk::FALSE }; self
    }
}

impl VkObjectDiscardable for vk::Sampler {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_sampler(self, None);
        }
    }
}
// ----------------------------------------------------------------------------------------------
