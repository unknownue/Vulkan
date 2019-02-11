
use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::VkDevice;
use crate::ci::VulkanCI;
use crate::utils::shaderc::VkShaderCompiler;
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

    path : PathBuf,
    main : String,

    tag_name: String,
    shader_type: ShaderType,
    shader_stage: vk::ShaderStageFlags,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ShaderType {
    GLSLSource,
    SprivSource,
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

impl From<ShaderModuleCI> for vk::ShaderModuleCreateInfo {

    fn from(value: ShaderModuleCI) -> vk::ShaderModuleCreateInfo {
        value.ci
    }
}

impl ShaderModuleCI {

    pub fn from_glsl(stage: vk::ShaderStageFlags, path: impl AsRef<Path>, tag_name: &str) -> ShaderModuleCI {

        ShaderModuleCI::new(stage, ShaderType::GLSLSource, path, tag_name)
    }

    pub fn from_spriv(stage: vk::ShaderStageFlags, path: impl AsRef<Path>, tag_name: &str) -> ShaderModuleCI {

        ShaderModuleCI::new(stage, ShaderType::SprivSource, path, tag_name)
    }

    fn new(stage: vk::ShaderStageFlags, ty: ShaderType, path: impl AsRef<Path>, tag_name: &str) -> ShaderModuleCI {

        ShaderModuleCI {
            ci: ShaderModuleCI::default_ci(),
            path: PathBuf::from(path.as_ref()),
            main: String::from("main"),
            tag_name: tag_name.into(),
            shader_type : ty,
            shader_stage: stage,
        }
    }

    pub fn main(mut self, name: impl AsRef<str>) -> ShaderModuleCI {
        self.main = String::from(name.as_ref()); self
    }

    pub fn flags(mut self, flags: vk::ShaderModuleCreateFlags) -> ShaderModuleCI {
        self.ci.flags = flags; self
    }

    pub fn build(self, device: &VkDevice, compiler: &mut VkShaderCompiler) -> VkResult<vk::ShaderModule> {

        let codes = match self.shader_type {
            | ShaderType::GLSLSource => {

                let source = load_to_string(self.path)?;
                compiler.compile_source_into_spirv(&source, self.shader_stage, &self.tag_name, &self.main)?
            },
            | ShaderType::SprivSource => {
                load_spriv_bytes(self.path)?
            },
        };

        let shader_module_ci = vk::ShaderModuleCreateInfo {
            code_size : codes.len(),
            p_code    : codes.as_ptr() as _,
            ..self.ci
        };

        let module = unsafe {
            device.logic.handle.create_shader_module(&shader_module_ci, None)
                .or(Err(VkError::create("Shader Module")))?
        };
        Ok(module)
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

impl From<ShaderStageCI> for vk::PipelineShaderStageCreateInfo {

    fn from(value: ShaderStageCI) -> vk::PipelineShaderStageCreateInfo {
        value.ci
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

    pub fn main(mut self, name: impl AsRef<str>) -> ShaderStageCI {
        self.main = CString::new(name.as_ref().to_owned())
            .expect("Invalid name of main func in shader."); self
    }

    pub fn flags(mut self, flags: vk::PipelineShaderStageCreateFlags) -> ShaderStageCI {
        self.ci.flags = flags; self
    }

    pub fn specialization(mut self, info: vk::SpecializationInfo) -> ShaderStageCI {
        self.specialization = Some(info); self
    }

    pub fn build(&self) -> vk::PipelineShaderStageCreateInfo {

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

impl crate::context::VulkanObject for vk::ShaderModule {

    fn discard(self, device: &VkDevice) {
        unsafe {
            device.logic.handle.destroy_shader_module(self, None);
        }
    }
}
// ---------------------------------------------------------------------------------------------------


// helper functions. ---------------------------------------------------------------------------------
fn load_spriv_bytes(path: PathBuf) -> VkResult<Vec<u8>> {

    let file = File::open(path.clone())
        .map_err(|_| VkError::path(path))?;
    let bytes = file.bytes()
        .filter_map(|byte| byte.ok())
        .collect();

    Ok(bytes)
}

fn load_to_string(path: PathBuf) -> VkResult<String> {

    let mut file = File::open(path.clone())
        .map_err(|_| VkError::path(path))?;
    let mut contents = String::new();
    let _size = file.read_to_string(&mut contents)
        .or(Err(VkError::other("Unable to shader code.")))?;

    Ok(contents)
}
// ---------------------------------------------------------------------------------------------------
