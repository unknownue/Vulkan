
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

    ci: vk::ShaderModuleCreateInfo,

    main : String,
    shader_type: ShaderType,
    shader_stage: vk::ShaderStageFlags,
}

#[derive(Debug, Clone)]
enum ShaderType {
    GLSLSource(Vec<u8>),
    SprivSource(PathBuf),
}

impl VulkanCI for ShaderModuleCI {
    type CIType = vk::ShaderModuleCreateInfo;

    fn default_ci() -> Self::CIType {

        vk::ShaderModuleCreateInfo {
            s_type    : vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next    : ptr::null(),
            flags     : vk::ShaderModuleCreateFlags::empty(),
            code_size : 0,
            p_code    : ptr::null(),
        }
    }
}

impl VkObjectBuildableCI for ShaderModuleCI {
    type ObjectType = vk::ShaderModule;

    fn build(&self, device: &VkDevice) -> VkResult<Self::ObjectType> {

        macro_rules! build_module {
            ($codes:ident) => {

                {
                    let shader_module_ci = vk::ShaderModuleCreateInfo {
                        code_size : $codes.len(),
                        p_code    : $codes.as_ptr() as _,
                        ..self.ci
                    };

                    unsafe {
                        device.logic.handle.create_shader_module(&shader_module_ci, None)
                            .or(Err(VkError::create("Shader Module")))?
                    }
                }
            };
        }

        let module = match &self.shader_type {
            | ShaderType::GLSLSource(codes) => {
                build_module!(codes)
            },
            | ShaderType::SprivSource(path) => {
                let codes = load_spriv_bytes(path)?;
                build_module!(codes)
            },
        };

        Ok(module)
    }
}

impl ShaderModuleCI {

    pub fn from_glsl(stage: vk::ShaderStageFlags, codes: Vec<u8>) -> ShaderModuleCI {

        ShaderModuleCI::inner_new(stage, ShaderType::GLSLSource(codes))
    }

    pub fn from_spriv(stage: vk::ShaderStageFlags, path: impl AsRef<Path>) -> ShaderModuleCI {

        ShaderModuleCI::inner_new(stage, ShaderType::SprivSource(PathBuf::from(path.as_ref())))
    }

    fn inner_new(stage: vk::ShaderStageFlags, ty: ShaderType) -> ShaderModuleCI {

        ShaderModuleCI {
            ci: ShaderModuleCI::default_ci(),
            main: String::from("main"),
            shader_type : ty,
            shader_stage: stage,
        }
    }

    #[inline(always)]
    pub fn main(mut self, name: impl AsRef<str>) -> ShaderModuleCI {
        self.main = String::from(name.as_ref()); self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::ShaderModuleCreateFlags) -> ShaderModuleCI {
        self.ci.flags = flags; self
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

    ci: vk::PipelineShaderStageCreateInfo,

    main: CString,
    specialization: Option<vk::SpecializationInfo>,
}

impl VulkanCI for ShaderStageCI {
    type CIType = vk::PipelineShaderStageCreateInfo;

    fn default_ci() -> Self::CIType {

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

impl ShaderStageCI {

    pub fn new(stage: vk::ShaderStageFlags, module: vk::ShaderModule) -> ShaderStageCI {

        ShaderStageCI {
            ci: vk::PipelineShaderStageCreateInfo {
                stage, module,
                ..ShaderStageCI::default_ci()
            },
            main: CString::new("main").unwrap(),
            specialization: None,
        }
    }

    #[inline(always)]
    pub fn main(mut self, name: impl AsRef<str>) -> ShaderStageCI {
        self.main = CString::new(name.as_ref().to_owned())
            .expect("Invalid name of main func in shader."); self
    }

    #[inline(always)]
    pub fn flags(mut self, flags: vk::PipelineShaderStageCreateFlags) -> ShaderStageCI {
        self.ci.flags = flags; self
    }

    #[inline(always)]
    pub fn specialization(mut self, info: vk::SpecializationInfo) -> ShaderStageCI {
        self.specialization = Some(info); self
    }

    pub fn value(&self) -> vk::PipelineShaderStageCreateInfo {

        let specialization = self.specialization
            .and_then(|s| Some(&s as *const vk::SpecializationInfo))
            .unwrap_or(ptr::null());

        vk::PipelineShaderStageCreateInfo {
            p_name: self.main.as_ptr(),
            p_specialization_info: specialization,
            ..self.ci
        }
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
