
pub use self::loader::GltfModelInfo;
pub use self::loader::load_gltf;
pub use self::asset::VkglTFModel;

pub use self::meshes::AttributeFlags;
pub use self::nodes::NodeAttachmentFlags;

mod loader;

mod scene;
mod material;

mod asset;
mod meshes;
mod nodes;
