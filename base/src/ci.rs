
pub mod shader;
pub mod pipeline;
pub mod image;
pub mod buffer;
pub mod memory;
pub mod command;
pub mod sync;

trait VulkanCI<T>
    where
        Self: Into<T> + Sized + Clone {

    fn default_ci() -> T;
}
