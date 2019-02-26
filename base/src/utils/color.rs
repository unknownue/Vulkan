
use crate::vkfloat;

#[derive(Debug, Clone, Copy)]
pub struct VkColor {
    pub r: vkfloat,
    pub g: vkfloat,
    pub b: vkfloat,
    pub a: vkfloat,
}

impl VkColor {
    pub const WHITE  : VkColor = VkColor::new(1.0, 1.0, 1.0, 1.0);
    pub const RED    : VkColor = VkColor::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN  : VkColor = VkColor::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE   : VkColor = VkColor::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW : VkColor = VkColor::new(1.0, 1.0, 0.0, 1.0);
    pub const PINK   : VkColor = VkColor::new(1.0, 0.0, 1.0, 1.0);
    pub const CYAN   : VkColor = VkColor::new(0.0, 1.0, 1.0, 1.0);
    pub const BLACK  : VkColor = VkColor::new(0.0, 0.0, 0.0, 1.0);

    pub const fn new(r: vkfloat, g: vkfloat, b: vkfloat, a: vkfloat) -> VkColor {
        VkColor { r, g, b, a }
    }

    pub fn new_u8(r: u8, g: u8, b: u8, a: u8) -> VkColor {
        VkColor {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0,
            a: (a as f32) / 255.0,
        }
    }
}

impl From<[vkfloat; 4]> for VkColor {

    fn from(color: [vkfloat; 4]) -> VkColor {
        VkColor {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        }
    }
}

impl From<VkColor> for [vkfloat; 4] {

    fn from(color: VkColor) -> [vkfloat; 4] {
        [color.r, color.g, color.b, color.a]
    }
}

impl From<[u8; 4]> for VkColor {

    fn from(color: [u8; 4]) -> VkColor {
        VkColor {
            r: color[0] as vkfloat / 255.0,
            g: color[1] as vkfloat / 255.0,
            b: color[2] as vkfloat / 255.0,
            a: color[3] as vkfloat / 255.0,
        }
    }
}

impl From<VkColor> for [u8; 4] {

    fn from(color: VkColor) -> [u8; 4] {
        [
            (color.r * 255.0).floor() as u8,
            (color.g * 255.0).floor() as u8,
            (color.b * 255.0).floor() as u8,
            (color.a * 255.0).floor() as u8,
        ]
    }
}
