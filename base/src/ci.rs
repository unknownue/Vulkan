
pub mod shader;
pub mod pipeline;
pub mod image;


trait VulkanCI<T>
    where
        Self: Into<T> + Sized + Clone {

    fn inner_default() -> Self;
}
