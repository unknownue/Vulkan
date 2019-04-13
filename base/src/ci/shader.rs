//! Types which simplify the creation of Vulkan shader objects.

use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};

use std::ffi::CString;
use std::ptr;

// ---------------------------------------------------------------------------------------------------
/// Wrapper class for `vk::ShaderModuleCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::ShaderModuleCreateInfo {
///     s_type    : vk::StructureType::SHADER_MODULE_CREATE_INFO,
///     p_next    : ptr::null(),
///     flags     : vk::ShaderModuleCreateFlags::empty(),
///     code_size : 0,
///     p_code    : ptr::null(),
/// }
/// ```
///
/// See [VkShaderModuleCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkShaderModuleCreateInfo.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct ShaderModuleCI {

    inner: vk::ShaderModuleCreateInfo,
    codes: Vec<u8>,
}

impl VulkanCI<vk::ShaderModuleCreateInfo> for ShaderModuleCI {

    fn default_ci() -> vk::ShaderModuleCreateInfo {

        vk::ShaderModuleCreateInfo {
            s_type    : vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next    : ptr::null(),
            flags     : vk::ShaderModuleCreateFlags::empty(),
            code_size : 0,
            p_code    : ptr::null(),
        }
    }
}

impl AsRef<vk::ShaderModuleCreateInfo> for ShaderModuleCI {

    fn as_ref(&self) -> &vk::ShaderModuleCreateInfo {
        &self.inner
    }
}

impl VkObjectBuildableCI for ShaderModuleCI {
    type ObjectType = vk::ShaderModule;

    /// Create `vk::ShaderModule` object, and return its handle.
    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let module = unsafe {
            device.logic.handle.create_shader_module(self.as_ref(), None)
                .or(Err(VkError::create("Shader Module")))?
        };

        Ok(module)
    }
}

impl ShaderModuleCI {

    /// Initialize `vk::ShaderModuleCreateInfo` with default value.
    ///
    /// `codes` must be valid SPIR-V code.
    pub fn new(codes: Vec<u8>) -> ShaderModuleCI {

        ShaderModuleCI {
            inner: vk::ShaderModuleCreateInfo {
                code_size: codes.len(),
                p_code   : codes.as_ptr() as _,
                ..ShaderModuleCI::default_ci()
            },
            codes,
        }
    }

    /// Set the `flags` member for `vk::ShaderModuleCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::ShaderModuleCreateFlags) -> ShaderModuleCI {
        self.inner.flags = flags; self
    }
}

impl crate::context::VkObjectDiscardable for vk::ShaderModule {

    fn discard_by(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_shader_module(self, None);
        }
    }
}
// ---------------------------------------------------------------------------------------------------

// ---------------------------------------------------------------------------------------------------
/// Wrapper class for `vk::PipelineShaderStageCreateInfo`.
///
/// The default values are defined as follows:
/// ``` ignore
/// vk::PipelineShaderStageCreateInfo {
///     s_type : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
///     p_next : ptr::null(),
///     flags  : vk::PipelineShaderStageCreateFlags::empty(),
///     p_name : ptr::null(),
///     stage  : vk::ShaderStageFlags::empty(),
///     module : vk::ShaderModule::null(),
///     p_specialization_info: ptr::null(),
/// }
/// ```
///
/// See [VkPipelineShaderStageCreateInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkPipelineShaderStageCreateInfo.html) for more detail.
///
#[derive(Debug, Clone)]
pub struct ShaderStageCI {

    inner: vk::PipelineShaderStageCreateInfo,

    main: CString,
    specialization: Option<vk::SpecializationInfo>,
}

impl VulkanCI<vk::PipelineShaderStageCreateInfo> for ShaderStageCI {

    fn default_ci() -> vk::PipelineShaderStageCreateInfo {

        vk::PipelineShaderStageCreateInfo {
            s_type : vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next : ptr::null(),
            flags  : vk::PipelineShaderStageCreateFlags::empty(),
            p_name : ptr::null(),
            stage  : vk::ShaderStageFlags::empty(),
            module : vk::ShaderModule::null(),
            p_specialization_info: ptr::null(),
        }
    }
}

impl AsRef<vk::PipelineShaderStageCreateInfo> for ShaderStageCI {

    fn as_ref(&self) -> &vk::PipelineShaderStageCreateInfo {
        &self.inner
    }
}

impl ShaderStageCI {

    /// Initialize `vk::PipelineShaderStageCreateInfo` with default value.
    ///
    /// `stage` indicates the pipeline stage of the shader module.
    ///
    /// `module` is the `vk::ShaderModule` handle created from shader codes.
    pub fn new(stage: vk::ShaderStageFlags, module: vk::ShaderModule) -> ShaderStageCI {

        let main = CString::new("main").unwrap();

        ShaderStageCI {
            inner: vk::PipelineShaderStageCreateInfo {
                stage, module,
                p_name: main.as_ptr(),
                ..ShaderStageCI::default_ci()
            },
            specialization: None,
            main,
        }
    }

    /// Set the `pName` member for `vk::PipelineShaderStageCreateInfo`.
    ///
    /// It specifies the entry point name of the shader. Default is `main`.
    #[inline(always)]
    pub fn main(mut self, name: impl AsRef<str>) -> ShaderStageCI {
        self.main = CString::new(name.as_ref().to_owned())
            .expect("Invalid name of main func in shader.");
        self.inner.p_name = self.main.as_ptr(); self
    }

    /// Set the `flags` member for `vk::PipelineShaderStageCreateInfo`.
    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineShaderStageCreateFlags) -> ShaderStageCI {
        self.inner.flags = flags; self
    }

    /// Set the `p_specialization_info` member for `vk::PipelineShaderStageCreateInfo`.
    ///
    /// It describes the specialization constants used in this shader stage.
    #[inline(always)]
    pub fn specialization(mut self, info: vk::SpecializationInfo) -> ShaderStageCI {
        self.specialization = Some(info);
        self.inner.p_specialization_info = &info; self
    }
}
// ---------------------------------------------------------------------------------------------------
