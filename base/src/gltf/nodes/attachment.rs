
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

pub struct AttachmentContent {
    pub transform: Option<Matrix4F>,
}

impl VkTryFrom<NodeAttachmentFlags> for NodeAttachments {

    fn try_from(flags: NodeAttachmentFlags) -> VkResult<NodeAttachments> {

        let element_size = flags.element_size()
            .ok_or(VkError::unimplemented("Node property combination"))?;
        let content = flags.new_transforms()
            .ok_or(VkError::unimplemented("Node property combination"))?;

        let result = NodeAttachments { element_size, content };
        Ok(result)
    }
}
// --------------------------------------------------------------------------------------

// --------------------------------------------------------------------------------------
pub trait AttachmentData {

    fn extend(&mut self, attachment: AttachmentContent);
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

            fn extend(&mut self, attachment: AttachmentContent) {

                let length = self.data.len();
                $(
                    read_transform!(self, attachment, $name_uniform, length, $attribute);
                )*
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
