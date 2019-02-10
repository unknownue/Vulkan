
use ash::vk;

use crate::error::{VkResult, VkError};

pub struct ShadercOptions {

    pub optimal_level : shaderc::OptimizationLevel,
    pub debug_info      : bool,
    pub suppress_warning: bool,
    pub error_warning   : bool,
}

impl Default for ShadercOptions {

    fn default() -> ShadercOptions {

        ShadercOptions {
            optimal_level    : shaderc::OptimizationLevel::Performance,
            debug_info       : true,
            suppress_warning : false,
            error_warning    : true,
        }
    }
}

impl ShadercOptions {

    fn to_shaderc_options(&self) -> VkResult<shaderc::CompileOptions> {

        // Default to compile target is vulkan and GLSL.
        let mut shaderc_options = shaderc::CompileOptions::new()
            .ok_or(VkError::shaderc("There are conflict in Shader Compile Options."))?;
        shaderc_options.set_optimization_level(self.optimal_level);

        if self.debug_info {
            shaderc_options.set_generate_debug_info();
        }
        if self.suppress_warning {
            shaderc_options.set_suppress_warnings();
        }
        if self.error_warning {
            shaderc_options.set_warnings_as_errors();
        }

        Ok(shaderc_options)
    }
}

pub struct VkShaderCompiler {

    compiler: shaderc::Compiler,
    options: ShadercOptions,
}

impl VkShaderCompiler {

    pub fn new() -> VkResult<VkShaderCompiler> {

        let compiler = shaderc::Compiler::new()
            .ok_or(VkError::shaderc("Failed to initialize shader compiler."))?;

        let target = VkShaderCompiler {
            compiler,
            options: ShadercOptions::default(),
        };
        Ok(target)
    }

    pub fn reset_compile_options(&mut self, options: ShadercOptions) {
        self.options = options;
    }

    pub fn compile_source_into_spirv(&mut self, source: &str, stage: vk::ShaderStageFlags, input_name: &str, entry_name: &str) -> VkResult<Vec<u8>> {

        let shader_kind = cast_shaderc_kind(stage);
        let compile_options = self.options.to_shaderc_options()?;

        let result = self.compiler.compile_into_spirv(source, shader_kind, input_name, entry_name, Some(&compile_options))
            .map_err(|e| VkError::shaderc(format!("Failed to compile {}({})", input_name, e)))?;

        if result.get_num_warnings() > 0 {
            println!("{}: {}", input_name, result.get_warning_messages());
        }

        let spirv = result.as_binary_u8().to_owned();
        Ok(spirv)
    }
}

fn cast_shaderc_kind(stage: vk::ShaderStageFlags) -> shaderc::ShaderKind {
    match stage {
        | vk::ShaderStageFlags::VERTEX                  => shaderc::ShaderKind::Vertex,
        | vk::ShaderStageFlags::GEOMETRY                => shaderc::ShaderKind::Geometry,
        | vk::ShaderStageFlags::TESSELLATION_CONTROL    => shaderc::ShaderKind::TessControl,
        | vk::ShaderStageFlags::TESSELLATION_EVALUATION => shaderc::ShaderKind::TessEvaluation,
        | vk::ShaderStageFlags::FRAGMENT                => shaderc::ShaderKind::Fragment,
        | vk::ShaderStageFlags::COMPUTE                 => shaderc::ShaderKind::Compute,
        | vk::ShaderStageFlags::ALL_GRAPHICS
        | _ => shaderc::ShaderKind::InferFromSource,
    }
}
