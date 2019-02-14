
use crate::gltf::nodes::node::Node;
use crate::error::{VkResult, VkError, VkTryFrom};
use crate::vkbytes;

use std::ops::{ BitAnd, BitOr, BitOrAssign, BitAndAssign };

type Matrix4F = nalgebra::Matrix4<f32>;

// --------------------------------------------------------------------------------------
pub struct NodeAttachments {

    /// the element size of each transform(including the padding).
    pub element_size: vkbytes,
    /// the transform data of this Node.
    pub content: Box<dyn AttachmentData>,
}

impl VkTryFrom<NodeAttachmentFlags> for NodeAttachments {

    fn try_from(flag: NodeAttachmentFlags) -> VkResult<NodeAttachments> {

        let element_size = flag.element_size()
            .ok_or(VkError::unimplemented("Node property combination"))?;
        let content = flag.new_transforms()
            .ok_or(VkError::unimplemented("Node property combination"))?;

        let result = NodeAttachments { element_size, content };
        Ok(result)
    }
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
pub trait AttachmentData {

    fn extend(&mut self, node: &Node);
}

macro_rules! read_transform {
    ($target:ident, $node:ident, $uniform_type:ident, transform) => {
        let new_uniforms = $uniform_type {
            transform: $node.transform().clone(),
            ..Default::default()
        };
        // println!("transform: {:?}", new_uniforms.transform);
        $target.data.push(new_uniforms);
    };
}

macro_rules! property_type {
    (transform) => (Matrix4F);
}

macro_rules! property_default {
    (transform) => { Matrix4F::identity() };
}

macro_rules! define_node_attachments {
    ($name_transform:ident, $name_uniform:ident, {
        $(
            $attribute:ident,
        )*
    }) => {

        #[allow(non_camel_case_types)]
        #[derive(Default)]
        pub(crate) struct $name_transform {
            data: Vec<$name_uniform>,
        }

        #[allow(non_camel_case_types)]
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

            fn extend(&mut self, node: &Node) {

                $(
                    read_transform!(self, node, $name_uniform, $attribute);
                )*
            }
        }
    };
}

define_node_attachments!(NA_T, NAUniform_T, {
    transform,
});
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct NodeAttachmentFlags(u32);

impl NodeAttachmentFlags {
    pub const NONE            : NodeAttachmentFlags = NodeAttachmentFlags(0b0);
    pub const TRANSFORM_MATRIX: NodeAttachmentFlags = NodeAttachmentFlags(0b1);
    // pub const JOINT_MATRIX    : NodeTransformFlags = NodeTransformFlags(0b10);

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
