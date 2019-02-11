
pub mod shader;
pub mod pipeline;
pub mod image;
pub mod buffer;

trait VulkanCI<T>
    where
        Self: Into<T> + Sized + Clone {

    fn inner_default() -> Self;
}
