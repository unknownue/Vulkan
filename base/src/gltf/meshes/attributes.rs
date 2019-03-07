
use ash::vk;

use crate::gltf::asset::GltfDocument;
use crate::ci::pipeline::VertexInputSCI;
use crate::error::{VkTryFrom, VkResult, VkError};
use crate::{vkbytes, vkptr};

use std::ops::{ BitAnd, BitOr, BitOrAssign, BitAndAssign };

// --------------------------------------------------------------------------------------
type Point3F  = nalgebra::Point3<f32>;
type Point2F  = nalgebra::Point2<f32>;

type Vector3F = nalgebra::Vector3<f32>;
type Vector4F = nalgebra::Vector4<f32>;
type Vector4U = nalgebra::Vector4<u16>;


pub struct AttributesData {

    /// the size of each vertices.
    pub vertex_size: vkbytes,
    /// the vertices attributes data of all primitive.
    pub data_content: Box<dyn VertexAttributes>,
}

impl VkTryFrom<AttributeFlags> for AttributesData {

    fn try_from(flags: AttributeFlags) -> VkResult<AttributesData> {

        let vertex_size = flags.vertex_size()
            .ok_or(VkError::unimplemented("Primitive attributes combination"))?;
        let content = flags.new_attributes()
            .ok_or(VkError::unimplemented("Primitive attributes combination"))?;

        let result = AttributesData { vertex_size, data_content: content };
        Ok(result)
    }
}

impl AttributesData {

    pub fn buffer_size_estimated(&self) -> vkbytes {
        (self.data_content.length() as vkbytes) * self.vertex_size
    }

    #[inline]
    pub fn input_descriptions(&self) -> VertexInputSCI {
        self.data_content.input_descriptions()
    }
}

pub struct AttributeExtendInfo {

    pub first_vertex: usize,
    pub vertex_count: usize,
}


// --------------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeFlags(u32);

impl AttributeFlags {
    pub const NONE      : AttributeFlags = AttributeFlags(0b0);
    pub const POSITION  : AttributeFlags = AttributeFlags(0b1);
    pub const NORMAL    : AttributeFlags = AttributeFlags(0b10);
    pub const TANGENT   : AttributeFlags = AttributeFlags(0b100);
    pub const TEXCOORD_0: AttributeFlags = AttributeFlags(0b1000);
    pub const TEXCOORD_1: AttributeFlags = AttributeFlags(0b10000);
    pub const COLOR_0   : AttributeFlags = AttributeFlags(0b100000);
    pub const JOINTS_0  : AttributeFlags = AttributeFlags(0b1000000);
    pub const WEIGHTS_0 : AttributeFlags = AttributeFlags(0b10000000);

    // POSITION.
    pub const ATTR_P: AttributeFlags = AttributeFlags(0b1);
    // POSITION, NORMAL.
    pub const ATTR_PN: AttributeFlags = AttributeFlags(0b11);
    // POSITION, TEXCOORD_0.
    pub const ATTR_PTE0: AttributeFlags = AttributeFlags(0b1001);
    // POSITION, NORMAL, TEXCOORD_0.
    pub const ATTR_PNTE0: AttributeFlags = AttributeFlags(0b1011);
    // POSITION, NORMAL, TANGENT, TEXCOORD_0, TEXCOORD_1, COLOR_0, JOINTS_0, WEIGHTS_0.
    pub const ATTR_ALL: AttributeFlags = AttributeFlags(0b11111111);

    fn vertex_size(&self) -> Option<vkbytes> {
        use std::mem::size_of;
        match *self {
            | AttributeFlags::ATTR_P     => Some(size_of::<Attr_P>()     as _),
            | AttributeFlags::ATTR_PN    => Some(size_of::<Attr_PN>()    as _),
            | AttributeFlags::ATTR_PTE0  => Some(size_of::<Attr_PTe0>()  as _),
            | AttributeFlags::ATTR_PNTE0 => Some(size_of::<Attr_PNTe0>() as _),
            | AttributeFlags::ATTR_ALL   => Some(size_of::<Attr_All>()   as _),
            | _ => None,
        }
    }

    fn new_attributes(&self) -> Option<Box<dyn VertexAttributes>> {
        match *self {
            | AttributeFlags::ATTR_P => {
                let attributes = Box::new(Attr_P::default());
                Some(attributes as Box<dyn VertexAttributes>)
            },
            | AttributeFlags::ATTR_PN => {
                let attributes = Box::new(Attr_PN::default());
                Some(attributes as Box<dyn VertexAttributes>)
            },
            | AttributeFlags::ATTR_PTE0 => {
                let attributes = Box::new(Attr_PTe0::default());
                Some(attributes as Box<dyn VertexAttributes>)
            },
            | AttributeFlags::ATTR_PNTE0 => {
                let attributes = Box::new(Attr_PNTe0::default());
                Some(attributes as Box<dyn VertexAttributes>)
            },
            | AttributeFlags::ATTR_ALL => {
                let attributes = Box::new(Attr_All::default());
                Some(attributes as Box<dyn VertexAttributes>)
            },
            | _ => None
        }
    }
}

impl BitAnd for AttributeFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        AttributeFlags(self.0 & rhs.0)
    }
}

impl BitAndAssign for AttributeFlags {

    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitOr for AttributeFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        AttributeFlags(self.0 | rhs.0)
    }
}

impl BitOrAssign for AttributeFlags {

    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
// --------------------------------------------------------------------------------------


// --------------------------------------------------------------------------------------
/// glTF Primitive attributes.
pub trait VertexAttributes {

    fn extend(&mut self, primitive: &gltf::Primitive, source: &GltfDocument) -> AttributeExtendInfo;

    fn length(&self) -> usize;

    fn map_data(&self, memory_ptr: vkptr);

    fn input_descriptions(&self) -> VertexInputSCI;
}

macro_rules! attribute_type {
    (position)   => (Point3F);
    (normal)     => (Vector3F);
    (tangents)   => (Vector4F);
    (texcoord_0) => (Point2F);
    (texcoord_1) => (Point2F);
    (color_0)    => (Vector4F);
    (joints_0)   => (Vector4U);
    (weights_0)  => (Vector4F);
}

macro_rules! attribute_default {
    (position)   => { Point3F::new(0.0, 0.0, 0.0) };
    (normal)     => { nalgebra::zero() };
    (tangents)   => { nalgebra::zero() };
    (texcoord_0) => { Point2F::new(0.0, 0.0) };
    (texcoord_1) => { Point2F::new(0.0, 0.0) };
    (color_0)    => { nalgebra::zero() };
    (joints_0)   => { nalgebra::zero() };
    (weights_0)  => { nalgebra::zero() };
}

macro_rules! attribute_format {
    (position)   => { vk::Format::R32G32B32_SFLOAT };
    (normal)     => { vk::Format::R32G32B32_SFLOAT };
    (tangents)   => { vk::Format::R32G32B32A32_SFLOAT };
    (texcoord_0) => { vk::Format::R32G32_SFLOAT };
    (texcoord_1) => { vk::Format::R32G32_SFLOAT };
    (color_0)    => { vk::Format::R32G32B32A32_SFLOAT };
    (joints_0)   => { vk::Format::R16G16B16A16_UNORM };
    (weights_0)  => { vk::Format::R32G32B32A32_SFLOAT };
}

macro_rules! read_attribute {
    ($target:ident, $reader:ident, $origin_length:ident, $VertexType:ident, position) => {

        if let Some(pos_iter) = $reader.read_positions() {

            if $target.data.len() == $origin_length {
                let vertex_iter = pos_iter.map(|pos| {
                    let position = Point3F::from(pos);
                    $VertexType { position, ..Default::default() }
                });
                $target.data.extend(vertex_iter);
            } else {
                for (i, pos) in pos_iter.enumerate() {
                    $target.data[i + $origin_length].position = Point3F::from(pos);
                }
            }
        }

    };
    ($target:ident, $reader:ident, $origin_length:ident, $VertexType:ident, normal) => {

        if let Some(normal_iter) = $reader.read_normals() {

            if $target.data.len() == $origin_length {
                let vertex_iter = normal_iter.map(|nor| {
                    let normal = Vector3F::from(nor);
                    $VertexType { normal, ..Default::default() }
                });
                $target.data.extend(vertex_iter);
            } else {
                for (i, normal) in normal_iter.enumerate() {
                    $target.data[i + $origin_length].normal = Vector3F::from(normal);
                }
            }
        }

    };
    ($target:ident, $reader:ident, $origin_length:ident, $VertexType:ident, tangents) => {

        if let Some(tangents_iter) = $reader.read_tangents() {

            if $target.data.len() == $origin_length {
                let vertex_iter = tangents_iter.map(|tan| {
                    let tangents = Vector4F::from(tan);
                    $VertexType { tangents, ..Default::default() }
                });
                $target.data.extend(vertex_iter);
            } else {
                for (i, tangent) in tangents_iter.enumerate() {
                    $target.data[i + $origin_length].tangents = Vector4F::from(tangent);
                }
            }
        }
    };
    ($target:ident, $reader:ident, $origin_length:ident, $VertexType:ident, texcoord_0) => {

        if let Some(texcoord_0_iter) = $reader.read_tex_coords(0) {

            if $target.data.len() == $origin_length {
                let vertex_iter = texcoord_0_iter.into_f32().map(|texcoord| {
                    let texcoord_0 = Point2F::from(texcoord);
                    $VertexType { texcoord_0, ..Default::default() }
                });
                $target.data.extend(vertex_iter);
            } else {
                for (i, texcoord_0) in texcoord_0_iter.into_f32().enumerate() {
                    $target.data[i + $origin_length].texcoord_0 = Point2F::from(texcoord_0);
                }
            }
        }
    };
    ($target:ident, $reader:ident, $origin_length:ident, $VertexType:ident, texcoord_1) => {

        if let Some(texcoord_1_iter) = $reader.read_tex_coords(1) {

            if $target.data.len() == $origin_length {
                let vertex_iter = texcoord_1_iter.into_f32().map(|texcoord| {
                    let texcoord_1 = Point2F::from(texcoord);
                    $VertexType { texcoord_1, ..Default::default() }
                });
                $target.data.extend(vertex_iter);
            } else {
                for (i, texcoord_1) in texcoord_1_iter.into_f32().enumerate() {
                    $target.data[i + $origin_length].texcoord_1 = Point2F::from(texcoord_1);
                }
            }
        }
    };
    ($target:ident, $reader:ident, $origin_length:ident, $VertexType:ident, color_0) => {

        if let Some(color_0_iter) = $reader.read_colors(0) {

            if $target.data.len() == $origin_length {
                let vertex_iter = color_0_iter.into_rgba_f32().map(|color| {
                    let color_0 = Vector4F::from(color);
                    $VertexType { color_0, ..Default::default() }
                });
                $target.data.extend(vertex_iter);
            } else {
                for (i, color_0) in color_0_iter.into_rgba_f32().enumerate() {
                    $target.data[i + $origin_length].color_0 = Vector4F::from(color_0);
                }
            }
        }
    };
    ($target:ident, $reader:ident, $origin_length:ident, $VertexType:ident, joints_0) => {

        if let Some(joints_0_iter) = $reader.read_joints(0) {

            if $target.data.len() == $origin_length {
                let vertex_iter = joints_0_iter.into_u16().map(|joint| {
                    let joints_0 = Vector4U::from(joint);
                    $VertexType { joints_0, ..Default::default() }
                });
                $target.data.extend(vertex_iter);
            } else {
                for (i, joints_0) in joints_0_iter.into_u16().enumerate() {
                    $target.data[i + $origin_length].joints_0 = Vector4U::from(joints_0);
                }
            }
        }
    };
    ($target:ident, $reader:ident, $origin_length:ident, $VertexType:ident, weights_0) => {

        if let Some(weights_0_iter) = $reader.read_weights(0) {

            if $target.data.len() == $origin_length {
                let vertex_iter = weights_0_iter.into_f32().map(|weight| {
                    let weights_0 = Vector4F::from(weight);
                    $VertexType { weights_0, ..Default::default() }
                });
                $target.data.extend(vertex_iter);
            } else {
                for (i, weights_0) in weights_0_iter.into_f32().enumerate() {
                    $target.data[i + $origin_length].weights_0 = Vector4F::from(weights_0);
                }
            }
        }
    };
}

macro_rules! define_attributes {
    ($name_attributes:ident, $name_vertex:ident, {
        $(
            $attribute:ident,
        )*
    }) => {

        #[allow(non_camel_case_types)]
        #[derive(Default)]
        struct $name_attributes {
            data: Vec<$name_vertex>,
        }

        #[allow(non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, Clone, Copy)]
        struct $name_vertex {
            $(
                $attribute: attribute_type!($attribute),
            )*
        }

        impl Default for $name_vertex {

            fn default() -> $name_vertex {
                $name_vertex {
                    $(
                        $attribute: attribute_default!($attribute),
                    )*
                }
            }
        }

        impl VertexAttributes for $name_attributes {

            fn extend(&mut self, primitive: &gltf::Primitive, source: &GltfDocument) -> AttributeExtendInfo {

                let reader = primitive.reader(|b| Some(&source.buffers[b.index()]));
                let start_vertex_index = self.data.len();

                $(
                    read_attribute!(self, reader, start_vertex_index, $name_vertex, $attribute);
                )*

                AttributeExtendInfo {
                    first_vertex: start_vertex_index,
                    vertex_count: self.data.len() - start_vertex_index,
                }
            }

            fn length(&self) -> usize {
                self.data.len()
            }

            fn map_data(&self, memory_ptr: vkptr) {

                unsafe {
                    (memory_ptr as vkptr<$name_vertex>).copy_from(self.data.as_ptr(), self.data.len());
                }
            }

            fn input_descriptions(&self) -> VertexInputSCI {

                let input_binding = vk::VertexInputBindingDescription {
                    binding: 0,
                    stride : ::std::mem::size_of::<$name_vertex>() as _,
                    input_rate: vk::VertexInputRate::VERTEX,
                };

                let mut sci = VertexInputSCI::new()
                    .add_binding(input_binding);

                $(
                    sci = sci.add_attribute(vk::VertexInputAttributeDescription {
                        location: 0,
                        binding : 0,
                        format  : attribute_format!($attribute),
                        offset  : memoffset::offset_of!($name_vertex, $attribute) as _,
                    });
                )*

                sci.inner_set_attribute_locations();

                sci
            }
        }
    };
}

// glTF Primitive with only position attribute.
define_attributes!(Attr_P, AttrVertex_P, { position, });

/// glTF Primitive with position and normal attributes.
define_attributes!(Attr_PN, AttrVertexPN, { position, normal, });

/// glTF Primitive with position and normal attributes.
define_attributes!(Attr_PTe0, AttrVertexPTe0, { position, texcoord_0, });

/// glTF Primitive with position, normal and texcoord_0 attributes.
define_attributes!(Attr_PNTe0, AttrVertex_PNTe0, { position, normal, texcoord_0, });

/// glTF Primitive with all attributes.
define_attributes!(Attr_All, AttrVertex_Ultimate, { position, normal, tangents, texcoord_0, texcoord_1, color_0, joints_0, weights_0, });
// --------------------------------------------------------------------------------------
