
mod physical;
mod logical;
mod queue;

pub use self::logical::{VkLogicalDevice, VkQueue, LogicDevConfig};
pub use self::physical::{VkPhysicalDevice, PhysicalDevConfig};


pub struct VkDevice {

    pub logic : logical::VkLogicalDevice,
    pub phy   : physical::VkPhysicalDevice,
}

impl VkDevice {

    #[inline]
    pub fn build<T>(&self, ci: T) -> crate::VkResult<T::ObjectType>
        where
            T: crate::ci::VkObjectBuildableCI {
        ci.build(self)
    }

    #[inline]
    pub fn discard(&self, object: impl VkObjectCreatable) {
        object.discard(self);
    }

    #[inline]
    pub fn free<T>(&self, object: T, pool: T::AllocatePool)
        where
            T: VkObjectAllocatable {

        object.free(self, pool);
    }
}

pub trait VkObjectCreatable: Copy {

    fn discard(self, device: &VkDevice);
}

pub trait VkObjectAllocatable: Copy {
    type AllocatePool: Copy;

    fn free(self, device: &VkDevice, pool: Self::AllocatePool);
}
