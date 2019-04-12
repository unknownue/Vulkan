
use crate::gltf::asset::ReferenceIndex;
use crate::error::{VkResult, VkError};
use crate::{vkbytes, vkptr};

use std::ops::{BitAnd, BitOr, BitOrAssign, BitAndAssign};
use std::collections::HashMap;
use std::convert::TryFrom;

type Matrix4F = nalgebra::Matrix4<f32>;

// --------------------------------------------------------------------------------------
pub struct NodeAttachments {

    /// the element size of each node attachments(including the padding).
    pub element_size: vkbytes,
    /// the attachment data of this Node.
    pub data_content: Box<dyn AttachmentData>,
    /// Map the json index of Node to the position index of its attachment data(in `data_content`).
    pub attachments_mapping: HashMap<ReferenceIndex, usize>,
}

pub struct AttachmentContent {
    pub transform: Option<Matrix4F>,
}

impl TryFrom<NodeAttachmentFlags> for NodeAttachments {
    type Error = VkError;

    fn try_from(flags: NodeAttachmentFlags) -> VkResult<NodeAttachments> {

        let element_size = flags.element_size()
            .ok_or(VkError::unimplemented("Node property combination"))?;
        let content = flags.new_transforms()
            .ok_or(VkError::unimplemented("Node property combination"))?;

        let result = NodeAttachments {
            element_size,
            data_content : content,
            attachments_mapping: HashMap::new(),
        };
        Ok(result)
    }
}

impl NodeAttachments {

    pub fn extend(&mut self, node_index: ReferenceIndex, attachment: AttachmentContent) {

        let attachment_position = self.data_content.extend(attachment);
        self.attachments_mapping.insert(node_index, attachment_position);
    }
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
pub trait AttachmentData {

    fn extend(&mut self, attachment: AttachmentContent) -> usize;

    fn length(&self) -> usize;

    fn map_data(&self, memory_ptr: vkptr, block_size: vkbytes, alignment: vkbytes);
}

macro_rules! property_type {
    (transform) => (Matrix4F);
}

macro_rules! property_default {
    (transform) => { Matrix4F::identity() };
}

macro_rules! read_transform {
    ($target:ident, $content:ident, $attachment_type:ident, $length:ident, transform) => {

        let transform_data = $content.transform.unwrap_or(property_default!(transform));

        if $target.data.len() == $length {

            let attachment = $attachment_type {
                transform: transform_data,
                ..Default::default()
            };
            // println!("transform: {:?}", new_uniforms.transform);
            $target.data.push(attachment);
        } else {
            $target.data[$length].transform = transform_data;
        }
    };
}

macro_rules! define_node_attachments {
    ($name_transform:ident, $name_uniform:ident, {
        $(
            $attribute:ident,
        )*
    }) => {

        #[allow(non_camel_case_types)]
        #[derive(Default)]
        struct $name_transform {
            data: Vec<$name_uniform>,
        }

        #[allow(non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, Clone, Copy)]
        struct $name_uniform {
            $(
                $attribute: property_type!($attribute),
            )*
        }

        impl Default for $name_uniform {

            fn default() -> $name_uniform {
                $name_uniform {
                    $(
                        $attribute: property_default!($attribute),
                    )*
                }

            }
        }

        impl AttachmentData for $name_transform {

            fn extend(&mut self, attachment: AttachmentContent) -> usize {

                let length = self.data.len();
                $(
                    read_transform!(self, attachment, $name_uniform, length, $attribute);
                )*

                length
            }

            fn length(&self) -> usize {
                self.data.len()
            }

            fn map_data(&self, memory_ptr: vkptr, block_size: vkbytes, alignment: vkbytes) {

                let mut vert_align = unsafe {
                    ash::util::Align::new(memory_ptr, alignment, block_size)
                };

                vert_align.copy_from_slice(&self.data);
            }
        }
    };
}

define_node_attachments!(NA_T, NAttachment_T, {
    transform,
});
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct NodeAttachmentFlags(u32);

impl NodeAttachmentFlags {
    pub const NONE            : NodeAttachmentFlags = NodeAttachmentFlags(0b0);
    pub const TRANSFORM_MATRIX: NodeAttachmentFlags = NodeAttachmentFlags(0b1);
    // pub const JOINT_MATRIX    : NodeAttachmentFlags = NodeAttachmentFlags(0b10);

    pub const NAF_T: NodeAttachmentFlags = NodeAttachmentFlags(0b1);

    fn element_size(&self) -> Option<vkbytes> {
        use std::mem::size_of;
        match *self {
            | NodeAttachmentFlags::NAF_T => Some(size_of::<NA_T>() as _),
            | _ => None,
        }
    }

    fn new_transforms(&self) -> Option<Box<dyn AttachmentData>> {
        match *self {
            | NodeAttachmentFlags::TRANSFORM_MATRIX => {
                let attachments = NA_T::default();
                Some(Box::new(attachments) as Box<dyn AttachmentData>)
            },
            | _ => None,
        }
    }
}

impl BitAnd for NodeAttachmentFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        NodeAttachmentFlags(self.0 & rhs.0)
    }
}

impl BitAndAssign for NodeAttachmentFlags {

    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOr for NodeAttachmentFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        NodeAttachmentFlags(self.0 | rhs.0)
    }
}

impl BitOrAssign for NodeAttachmentFlags {

    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
// --------------------------------------------------------------------------------------
