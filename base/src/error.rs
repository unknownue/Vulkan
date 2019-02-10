
use failure::{ Backtrace, Context, Fail };

use std::result;
use std::path::{ Path, PathBuf };
use std::fmt;

pub type VkResult<T> = result::Result<T, VkError>;

// -------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct VkError {

    ctx: Context<VkErrorKind>,
}

impl VkError {

    pub fn kind(&self) -> &VkErrorKind {
        self.ctx.get_context()
    }

    pub fn unlink(target_name: &'static str) -> VkError {
        VkError::from(VkErrorKind::Unlink { target_name })
    }

    pub fn query(query_target: &'static str) -> VkError {
        VkError::from(VkErrorKind::Query { query_target })
    }

    pub fn create(create_target: &'static str) -> VkError {
        VkError::from(VkErrorKind::Create { create_target })
    }

    pub fn unsupported(feature: &'static str) -> VkError {
        VkError::from(VkErrorKind::UnSupport { feature })
    }

    pub fn device(ops_description: &'static str) -> VkError {
        VkError::from(VkErrorKind::Device { ops_description })
    }

    pub fn shaderc(compile_message: impl AsRef<str>) -> VkError {
        VkError::from(VkErrorKind::Shaderc {
            compile_message: compile_message.as_ref().to_string()
        })
    }

    pub(crate) fn window(description: impl AsRef<str>) -> VkError {
        VkError::from(VkErrorKind::Window { description: description.as_ref().to_string() })
    }

    /// A convenience routine for creating an error associated with a path.
    pub fn path(path: impl AsRef<Path>)-> VkError {
        VkError::from(VkErrorKind::Path { path: path.as_ref().to_path_buf() })
    }

    pub fn other(description: impl AsRef<str>) -> VkError {
        VkError::from(VkErrorKind::Other {
            description: description.as_ref().to_string()
        })
    }
}

impl Fail for VkError {

    fn cause(&self) -> Option<&Fail> {
        self.ctx.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl fmt::Display for VkError {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ctx.fmt(f)
    }
}
// -------------------------------------------------------------------------------------------

// -------------------------------------------------------------------------------------------
/// The specific kind of error that can occur.
#[derive(Debug, Fail)]
pub enum VkErrorKind {

    /// An error occurred while building connection between application and Vulkan.
    #[fail(display = "Failed to bridge connection between {} and Vulkan.", target_name)]
    Unlink { target_name: &'static str },
    /// An error occurred while querying some properties from Vulkan.
    #[fail(display = "Failed to query {} property from Vulkan or Device.", query_target)]
    Query { query_target: &'static str },
    /// An error occurred while creating Vulkan Object.
    #[fail(display = "Failed to create {}.", create_target)]
    Create { create_target: &'static str },
    /// An error indicated requiring some unsupported feature.
    #[fail(display = "Feature {} is not supported in current Vulkan Device.", feature)]
    UnSupport { feature: &'static str },
    /// An error triggered by Invalid Device operations.
    #[fail(display = "Invalid Operation: {}", ops_description)]
    Device { ops_description: &'static str },
    /// An error that occurred while trying to compile shader code in runtime.
    #[fail(display = "Error occurred during runtime shader compiling: {}.", compile_message)]
    Shaderc { compile_message: String },
    /// An error occurred while communicate with Window.
    #[fail(display = "Failed to interact with Window: {}.", description)]
    Window { description: String },
    /// An error that occurred while working with a file path.
    #[fail(display = "Failed to locate file at: {:?}", path)]
    Path { path: PathBuf },
    /// Other errors.
    #[fail(display = "{}", description)]
    Other { description: String },
}

impl From<VkErrorKind> for VkError {

    fn from(kind: VkErrorKind) -> VkError {
        VkError::from(Context::new(kind))
    }
}

impl From<Context<VkErrorKind>> for VkError {

    fn from(ctx: Context<VkErrorKind>) -> VkError {
        VkError { ctx }
    }
}
// -------------------------------------------------------------------------------------------
