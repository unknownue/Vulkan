
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::ci::{VulkanCI, VkObjectBuildableCI};
use crate::error::{VkResult, VkError};

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::ffi::CString;
use std::ptr;

// ---------------------------------------------------------------------------------------------------
/// Wrapper class for vk::ShaderModuleCreateInfo.
#[derive(Debug, Clone)]
pub struct ShaderModuleCI {

    inner: vk::ShaderModuleCreateInfo,

    codes: Vec<u8>,
    shader_stage: vk::ShaderStageFlags,
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

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        let module = unsafe {
            device.logic.handle.create_shader_module(self.as_ref(), None)
                .or(Err(VkError::create("Shader Module")))?
        };

        Ok(module)
    }
}

impl ShaderModuleCI {

    pub fn from_glsl(stage: vk::ShaderStageFlags, codes: Vec<u8>) -> ShaderModuleCI {

        ShaderModuleCI::inner_new(stage, codes)
    }

    pub fn from_spriv(stage: vk::ShaderStageFlags, path: impl AsRef<Path>) -> VkResult<ShaderModuleCI> {

        let codes = load_spriv_bytes(&path.as_ref().to_path_buf())?;
        let ci = ShaderModuleCI::inner_new(stage, codes);
        Ok(ci)
    }

    fn inner_new(shader_stage: vk::ShaderStageFlags, codes: Vec<u8>) -> ShaderModuleCI {

        ShaderModuleCI {
            inner: vk::ShaderModuleCreateInfo {
                code_size: codes.len(),
                p_code   : codes.as_ptr() as _,
                ..ShaderModuleCI::default_ci()
            },
            codes, shader_stage,
        }
    }

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
/// Wrapper class for vk::PipelineShaderStageCreateInfo.
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

    #[inline(always)]
    pub fn main(mut self, name: impl AsRef<str>) -> ShaderStageCI {
        self.main = CString::new(name.as_ref().to_owned())
            .expect("Invalid name of main func in shader.");
        self.inner.p_name = self.main.as_ptr(); self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineShaderStageCreateFlags) -> ShaderStageCI {
        self.inner.flags = flags; self
    }

    #[inline(always)]
    pub fn specialization(mut self, info: vk::SpecializationInfo) -> ShaderStageCI {
        self.specialization = Some(info);
        self.inner.p_specialization_info = &info; self
    }
}
// ---------------------------------------------------------------------------------------------------


// helper functions. ---------------------------------------------------------------------------------
fn load_spriv_bytes(path: &PathBuf) -> VkResult<Vec<u8>> {

    let file = File::open(path.clone())
        .map_err(|_| VkError::path(path))?;
    let bytes = file.bytes()
        .filter_map(|byte| byte.ok())
        .collect();

    Ok(bytes)
}
// ---------------------------------------------------------------------------------------------------
