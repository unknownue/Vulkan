
/// Represent Vulkan Object used during the whole runtime of application.
///
/// These objects must be initialized and destroyed in a specific order, so they have to be destroyed manually.
pub trait VkBackendObject {

    unsafe fn discard(&self);
}
