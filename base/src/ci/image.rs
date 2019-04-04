//! Types which simplify the creation of Vulkan image objects.

use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::{VkDevice, VkObjectDiscardable, VkObjectBindable};
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};
use crate::{vkbytes, vkuint, vkfloat};

use std::ptr;

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::ImageCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::ImageCreateInfo {
///     s_type: vk::StructureType::IMAGE_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::ImageCreateFlags::empty(),
///     image_type: vk::ImageType::TYPE_2D,
///     format: vk::Format::UNDEFINED,
///     extent: Default::default(),
///     mip_levels  : 1,
///     array_layers: 1,
///     samples: vk::SampleCountFlags::TYPE_1,
///     tiling : vk::ImageTiling::OPTIMAL,
///     usage  : vk::ImageUsageFlags::empty(),
///     sharing_mode  : vk::SharingMode::EXCLUSIVE,
///     initial_layout: vk::ImageLayout::UNDEFINED,
///     queue_family_index_count: 0,
///     p_queue_family_indices  : ptr::null(),
/// }
/// ```
///
/// See [VkImageCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkImageCreateInfo.html) for more detail.
///
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

    /// Create `vk::Image` object, and return its handle and memory requirement.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        debug_assert_ne!(self.inner.usage, vk::ImageUsageFlags::empty(), "the usage member of vk::ImageCreateInfo must not be 0!");

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

    /// Initialize `vk::ImageCreateInfo` with default value.
    ///
    /// `type_` specifies the basic dimensionality of the image.
    ///
    /// `format` specifies the texel format of this image.
    ///
    /// `dimension` specifies dimension of the base level.
    pub fn new(type_: vk::ImageType, format: vk::Format, dimension: vk::Extent3D) -> ImageCI {

        debug_assert!(
            dimension.width > 0 && dimension.height > 0 && dimension.depth > 0,
            "The width, height and depth of image must be greater than 0!");

        ImageCI {
            inner: vk::ImageCreateInfo {
                image_type: type_,
                format,
                extent: dimension,
                ..ImageCI::default_ci()
            },
            queue_families: None,
        }
    }

    /// Convenient method to create a 2D `ImageCI`.
    ///
    /// `format` specifies the texel format of this image.
    ///
    /// `dimension` specifies dimension of the base level.
    pub fn new_2d(format: vk::Format, dimension: vk::Extent2D) -> ImageCI {

        let extent = vk::Extent3D {
            width : dimension.width,
            height: dimension.height,
            depth : 1, // depth is always 1 for vk::ImageType::TYPE_2D.
        };

        ImageCI::new(vk::ImageType::TYPE_2D, format, extent)
    }

    /// Set the `flags` member for `vk::ImageCreateInfo`.
    ///
    /// It describes additional parameters of the image.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::ImageCreateFlags) -> ImageCI {
        self.inner.flags = flags; self
    }

    /// Set the `usage` member for `vk::ImageCreateInfo`.
    ///
    /// It describes the intended usage of the image.
    #[inline(always)]
    pub fn usages(mut self, flags: vk::ImageUsageFlags) -> ImageCI {
        self.inner.usage = flags; self
    }

    /// Set the `tiling` member for `vk::ImageCreateInfo`.
    ///
    /// Set tiling to `vk::ImageTiling::OPTIMAL` for the most part.
    #[inline(always)]
    pub fn tiling(mut self, tiling: vk::ImageTiling) -> ImageCI {
        self.inner.tiling = tiling; self
    }

    /// Set the `samples` member for `vk::ImageCreateInfo`.
    ///
    /// It specifies the sample count for each pixel.
    #[inline(always)]
    pub fn samples(mut self, count: vk::SampleCountFlags) -> ImageCI {
        self.inner.samples = count; self
    }

    /// Set the `mip_levels` member for `vk::ImageCreateInfo`.
    ///
    /// It describes the number of mipmap level of this image.
    #[inline(always)]
    pub fn mip_levels(mut self, level: vkuint) -> ImageCI {
        debug_assert!(level > 0, "The mip_levels of image must be greater than 0!");
        self.inner.mip_levels = level; self
    }

    /// Set the `array_layers` member for `vk::ImageCreateInfo`.
    ///
    /// It describes the number of layers in this image.
    #[inline(always)]
    pub fn array_layers(mut self, layers: vkuint) -> ImageCI {
        debug_assert!(layers > 0, "The array_layers of image must be greater than 0!");
        self.inner.array_layers = layers; self
    }

    /// Set the `initial_layout` member for `vk::ImageCreateInfo`.
    ///
    /// It describes the initial `vk::ImageLayout` of this image.
    #[inline(always)]
    pub fn initial_layout(mut self, layout: vk::ImageLayout) -> ImageCI {
        self.inner.initial_layout = layout; self
    }

    /// Set the list of queue families that will access this image.
    ///
    /// The `sharing_mode` member of `vk::ImageCreateInfo` will be set to `vk::SharingMode::CONCURRENT` automatically.
    #[inline(always)]
    pub fn sharing_queues(mut self, families_indices: Vec<vkuint>) -> ImageCI {

        self.inner.queue_family_index_count = families_indices.len() as _;
        self.inner.p_queue_family_indices   = families_indices.as_ptr();

        debug_assert!(self.inner.queue_family_index_count > 1, "The number of shared queue families must be greater than 1!");

        self.queue_families = Some(families_indices);
        self.inner.sharing_mode = vk::SharingMode::CONCURRENT; self
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

    /// Bind a specific range of `memory` to this image.
    fn bind(self, device: &VkDevice, memory: vk::DeviceMemory, offset: vkbytes) -> VkResult<()> {
        unsafe {
            device.logic.handle.bind_image_memory(self, memory, offset)
                .map_err(|_| VkError::device("Binding Image Memory"))
        }
    }
}
// ----------------------------------------------------------------------------------------------

// ----------------------------------------------------------------------------------------------
/// Wrapper class for `vk::ImageViewCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::ImageViewCreateInfo {
///     s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::ImageViewCreateFlags::empty(),
///     image : vk::Image::null(),
///     view_type: vk::ImageViewType::TYPE_2D,
///     format: vk::Format::UNDEFINED,
///     components: vk::ComponentMapping {
///         r: vk::ComponentSwizzle::R,
///         g: vk::ComponentSwizzle::G,
///         b: vk::ComponentSwizzle::B,
///         a: vk::ComponentSwizzle::A,
///     },
///     subresource_range: vk::ImageSubresourceRange {
///         aspect_mask      : vk::ImageAspectFlags::COLOR,
///         base_mip_level   : 0,
///         level_count      : 1,
///         base_array_layer : 0,
///         layer_count      : 1,
///     },
/// }
/// ```
///
/// See [VkImageViewCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkImageViewCreateInfo.html) for more detail.
///
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

    /// Create `vk::ImageView` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let view = unsafe {
            device.logic.handle.create_image_view(self.as_ref(), None)
                .map_err(|_| VkError::create("Image View"))?
        };
        Ok(view)
    }
}

impl ImageViewCI {

    /// Initialize `vk::ImageCreateInfo` with default value.
    ///
    /// `image` specifies the image that this image view would access.
    ///
    /// `type_` specifies the type of this image view.
    ///
    /// `format` specifies what texel format would this image view interpret to.
    pub fn new(image: vk::Image, type_: vk::ImageViewType, format: vk::Format) -> ImageViewCI {

        ImageViewCI {
            inner: vk::ImageViewCreateInfo {
                image, format,
                view_type: type_,
                ..ImageViewCI::default_ci()
            },
        }
    }

    /// Set the `flags` member for `vk::ImageViewCreateInfo`.
    ///
    /// It describes additional parameters of the image view.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::ImageViewCreateFlags) -> ImageViewCI {
        self.inner.flags = flags; self
    }

    /// Set the `components` member for `vk::ImageViewCreateInfo`.
    ///
    /// It specifies the color remapping for this image.
    #[inline(always)]
    pub fn components(mut self, components: vk::ComponentMapping) -> ImageViewCI {
        self.inner.components = components;; self
    }

    /// Set the `subresource_range` member for `vk::ImageViewCreateInfo`.
    ///
    /// It specifies the levels and layers that this image view would access.
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
/// Wrapper class for `vk::ImageMemoryBarrier`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::ImageMemoryBarrier {
///     s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
///     p_next: ptr::null(),
///     src_access_mask: vk::AccessFlags::empty(),
///     dst_access_mask: vk::AccessFlags::empty(),
///     old_layout: vk::ImageLayout::UNDEFINED,
///     new_layout: vk::ImageLayout::UNDEFINED,
///     src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
///     dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
///     image: vk::Image::null(),
///     subresource_range: Default::default(),
/// }
/// ```
///
/// See [VkImageMemoryBarrier](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkImageMemoryBarrier.html) for more detail.
/////
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

    /// Initialize `vk::ImageMemoryBarrier` with default value.
    ///
    /// `image` is the image affected by this barrier.
    ///
    /// `subrange` specifies the subresource range affected by this barrier.
    pub fn new(image: vk::Image, subrange: vk::ImageSubresourceRange) -> ImageBarrierCI {

        ImageBarrierCI {
            inner: vk::ImageMemoryBarrier {
                image,
                subresource_range: subrange,
                ..ImageBarrierCI::default_ci()
            },
        }
    }

    /// Set the `src_access_mask` and `dst_access_mask` members for `vk::ImageViewCreateInfo`.
    #[inline(always)]
    pub fn access_mask(mut self, from: vk::AccessFlags, to: vk::AccessFlags) -> Self {
        self.inner.src_access_mask = from;
        self.inner.dst_access_mask = to; self
    }

    /// Set the `old_layout` and `new_layout` members for `vk::ImageViewCreateInfo`.
    ///
    /// It specifies the layout transition for the image.
    #[inline(always)]
    pub fn layout(mut self, from: vk::ImageLayout, to: vk::ImageLayout) -> Self {
        self.inner.old_layout = from;
        self.inner.new_layout = to; self
    }

    /// Set the `src_queue_family_index` and `dst_queue_family_index` members for `vk::ImageViewCreateInfo`.
    ///
    /// It specifies the queue family ownership transfer for the image.
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
/// Wrapper class for `vk::SamplerCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::SamplerCreateInfo {
///     s_type: vk::StructureType::SAMPLER_CREATE_INFO,
///     p_next: ptr::null(),
///     flags : vk::SamplerCreateFlags::empty(),
///     mag_filter: vk::Filter::LINEAR,
///     min_filter: vk::Filter::LINEAR,
///     mipmap_mode: vk::SamplerMipmapMode::LINEAR,
///     address_mode_u: vk::SamplerAddressMode::REPEAT,
///     address_mode_v: vk::SamplerAddressMode::REPEAT,
///     address_mode_w: vk::SamplerAddressMode::REPEAT,
///     anisotropy_enable: vk::FALSE,
///     max_anisotropy   : 1.0,
///     compare_enable: vk::FALSE,
///     compare_op    : vk::CompareOp::ALWAYS,
///     mip_lod_bias: 0.0,
///     min_lod     : 0.0,
///     max_lod     : 0.0,
///     border_color: vk::BorderColor::INT_OPAQUE_BLACK,
///     unnormalized_coordinates: vk::FALSE,
/// }
/// ```
///
/// See [VkSamplerCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkSamplerCreateInfo.html) for more detail.
/////
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

impl VkObjectBuildableCI for SamplerCI {
    type ObjectType = vk::Sampler;

    /// Create `vk::Sampler` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let sampler = unsafe {
            device.logic.handle.create_sampler(self.as_ref(), None)
                .map_err(|_| VkError::create("Sampler"))?
        };
        Ok(sampler)
    }
}

impl AsRef<vk::SamplerCreateInfo> for SamplerCI {

    fn as_ref(&self) -> &vk::SamplerCreateInfo {
        &self.inner
    }
}

impl SamplerCI {

    /// Initialize `vk::SamplerCreateInfo` with default value.
    #[inline(always)]
    pub fn new() -> SamplerCI {
        SamplerCI {
            inner: SamplerCI::default_ci(),
        }
    }

    /// Set the `mag_filter` and `min_filter` members for `vk::SamplerCreateInfo`.
    ///
    /// `mag` specifies the magnification filter to apply to lookups.
    ///
    /// `min` specifies the minification filter to apply to lookups.
    #[inline(always)]
    pub fn filter(mut self, mag: vk::Filter, min: vk::Filter) -> SamplerCI {
        self.inner.mag_filter = mag;
        self.inner.min_filter = min; self
    }

    /// Set the `mipmap_mode` member for `vk::SamplerCreateInfo`.
    ///
    /// `mode` specifies the mipmap filter to apply to lookups.
    #[inline(always)]
    pub fn mipmap(mut self, mode: vk::SamplerMipmapMode) -> SamplerCI {
        self.inner.mipmap_mode = mode; self
    }

    /// Set the `address_mode_u`, `address_mode_v` and `address_mode_w` members for `vk::SamplerCreateInfo`.
    ///
    /// `u`, `v` and `w` specifies the addressing mode for outside [0..1] range for U, V, W coordinate.
    #[inline(always)]
    pub fn address(mut self, u: vk::SamplerAddressMode, v: vk::SamplerAddressMode, w: vk::SamplerAddressMode) -> SamplerCI {
        self.inner.address_mode_u = u;
        self.inner.address_mode_v = v;
        self.inner.address_mode_w = w; self
    }

    /// Set the `mip_bias`, `min` and `max` members for `vk::SamplerCreateInfo`.
    ///
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

    /// Set the `max_anisotropy` member for `vk::SamplerCreateInfo`.
    ///
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

    /// Set the `compare_op` member for `vk::SamplerCreateInfo`.
    ///
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

    /// Set the `border_color` member for `vk::SamplerCreateInfo`.
    ///
    /// `border_color` specifies the predefined border color to use.
    #[inline(always)]
    pub fn border_color(mut self, color: vk::BorderColor) -> SamplerCI {
        self.inner.border_color = color; self
    }

    /// Set the `unnormalized_coordinates` member for `vk::SamplerCreateInfo`.
    ///
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
