
pub use self::workflow::{RenderWorkflow, WindowContext, WindowConfig};
pub use self::workflow::ProcPipeline;
pub use self::error::{VkResult, VkError, VkErrorKind};
pub use self::utils::frame::FrameAction;
pub use self::input::EventController;
pub use self::camera::FlightCamera;

pub mod context;
pub mod ci;
pub mod utils;
pub mod command;
pub mod platforms;
pub mod gltf;
pub mod texture;
pub mod ui;

mod error;
mod camera;
mod workflow;
mod input;

// type alias ------------------------------------
/// unsigned integer type commonly used in vulkan(an alias type of uint32_t in C++).
#[allow(non_camel_case_types)]
pub type vkuint = u32;
/// signed integer type used in vulkan(an alias type of int32_t in C++).
#[allow(non_camel_case_types)]
pub type vksint = i32;
/// float type used in vulkan.
#[allow(non_camel_case_types)]
pub type vkfloat = ::std::os::raw::c_float;
/// unsigned long integer type used in vulkan.
#[allow(non_camel_case_types)]
pub type vklint = u64;
/// char type used in vulkan.
#[allow(non_camel_case_types)]
pub type vkchar = ::std::os::raw::c_char;
/// boolean type used in vulkan(an alias type of VkBool32 in C++).
#[allow(non_camel_case_types)]
pub type vkbool = ash::vk::Bool32;
#[allow(non_camel_case_types)]
// raw pointer type used in vulkan.(the size of c_void in bytes is 1).
pub type vkptr<T=::std::os::raw::c_void> = *mut T;
/// the number of bytes(an alias type of VkDeviceSize in C++), used to measure the size of memory block(buffer, image...).
#[allow(non_camel_case_types)]
pub type vkbytes = ash::vk::DeviceSize;

// type alias for point, vector and matrix.
pub type Point2F  = nalgebra::Point2<f32>;
pub type Point3F  = nalgebra::Point3<f32>;
pub type Point4F  = nalgebra::Point4<f32>;
pub type Vector3F = nalgebra::Vector3<f32>;
pub type Vector4F = nalgebra::Vector4<f32>;
pub type Matrix4F = nalgebra::Matrix4<f32>;
// -----------------------------------------------
