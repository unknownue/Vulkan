
use ash::vk;

use crate::gltf::meshes::{MeshAsset, MeshResource, AttributeFlags};
use crate::gltf::nodes::{NodeAsset, NodeResource, NodeAttachmentFlags};
use crate::gltf::material::{MaterialAsset, MaterialResource};
use crate::gltf::scene::Scene;

use crate::command::{VkCmdRecorder, IGraphics};
use crate::context::{VkDevice, VmaResourceDiscardable};
use crate::Matrix4F;
use crate::error::{VkResult, VkTryFrom};

use std::collections::HashMap;

pub type ReferenceIndex = usize;
pub type   StorageIndex = usize;

// --------------------------------------------------------------------------------------
pub struct GltfDocument {
    pub doc: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images : Vec<gltf::image::Data>,

    pub transform: Option<Matrix4F>,
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
pub trait AssetAbstract: Sized {
    const ASSET_NAME: &'static str;

    fn read_doc(&mut self, source: &GltfDocument, scene: &Scene) -> VkResult<()>;
}

pub struct AssetElementList<T> {

    list: Vec<T>,
    query_table: HashMap<ReferenceIndex, StorageIndex>,
}

impl<T> Default for AssetElementList<T> {

    fn default() -> AssetElementList<T> {
        AssetElementList {
            list: Vec::new(),
            query_table: HashMap::new(),
        }
    }
}

impl<T> AssetElementList<T> {

    pub fn push(&mut self, ref_index: ReferenceIndex, element: T) {

        let storage_index = self.list.len();
        self.query_table.insert(ref_index, storage_index);

        self.list.push(element);
    }

    pub fn get(&self, ref_index: ReferenceIndex) -> &T {

        debug_assert!(self.query_table.contains_key(&ref_index));

        let storage_index = self.query_table.get(&ref_index).cloned().unwrap();
        &self.list[storage_index]
    }
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
pub struct AssetRepository {
    pub nodes : NodeAsset,
    pub meshes: MeshAsset,
    pub materials: MaterialAsset,
}

impl AssetRepository {

    pub fn new(attr_flag: AttributeFlags, attachment_flag: NodeAttachmentFlags) -> VkResult<AssetRepository> {

        let repository = AssetRepository {
            nodes : NodeAsset::try_from(attachment_flag)?,
            meshes: MeshAsset::try_from(attr_flag)?,
            materials: MaterialAsset::new()?,
        };
        Ok(repository)
    }

    pub fn allocate(self, device: &mut VkDevice, scene: Scene) -> VkResult<VkglTFModel> {

        let nodes_allocated  = self.nodes.allocate(device, device.phy.limits.min_uniform_buffer_offset_alignment)?;
        let meshes_allocated = self.meshes.allocate(device)?;

        let result = VkglTFModel {
            scene,
            meshes: meshes_allocated,
            nodes : nodes_allocated,
            materials: self.materials,
        };

        Ok(result)
    }
}
// --------------------------------------------------------------------------------------


// --------------------------------------------------------------------------------------
pub struct VkglTFModel {

    pub meshes: MeshResource,
    pub nodes : NodeResource,
    pub materials: MaterialResource,

    scene: Scene,
}

pub struct ModelRenderParams {

    pub descriptor_set : vk::DescriptorSet,
    pub pipeline_layout: vk::PipelineLayout,
    pub material_stage : Option<vk::ShaderStageFlags>,
}

impl VkglTFModel {

    pub fn record_command(&self, recorder: &VkCmdRecorder<IGraphics>, params: &ModelRenderParams) {

        self.meshes.record_command(recorder);
        self.scene.record_command(recorder, self, params);
    }
}

impl VmaResourceDiscardable for VkglTFModel {

    fn discard_by(self, vma: &mut vma::Allocator) -> VkResult<()> {

        self.meshes.discard_by(vma)?;
        self.nodes.discard_by(vma)
    }
}
// --------------------------------------------------------------------------------------
