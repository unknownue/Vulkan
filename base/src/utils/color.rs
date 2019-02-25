
use crate::vkfloat;

#[derive(Debug, Clone)]
pub struct VkColor {
    pub r: vkfloat,
    pub g: vkfloat,
    pub b: vkfloat,
    pub a: vkfloat,
}

impl VkColor {
    const WHITE  : VkColor = VkColor::new(1.0, 1.0, 1.0, 1.0);
    const RED    : VkColor = VkColor::new(1.0, 0.0, 0.0, 1.0);
    const GREEN  : VkColor = VkColor::new(0.0, 1.0, 0.0, 1.0);
    const BLUE   : VkColor = VkColor::new(0.0, 0.0, 1.0, 1.0);
    const YELLOW : VkColor = VkColor::new(1.0, 1.0, 0.0, 1.0);
    const PINK   : VkColor = VkColor::new(1.0, 0.0, 1.0, 1.0);
    const CYAN   : VkColor = VkColor::new(0.0, 1.0, 1.0, 1.0);
    const BLACK  : VkColor = VkColor::new(0.0, 0.0, 0.0, 1.0);

    pub const fn new(r: vkfloat, g: vkfloat, b: vkfloat, a: vkfloat) -> VkColor {
        VkColor { r, g, b, a }
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
