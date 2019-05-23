
use winit::VirtualKeyCode;

use crate::input::EventController;
use crate::{Vec3F, Mat4F};


/// A simple flight through camera.
///
/// This camera is mainly modified from https://learnopengl.com.
pub struct FlightCamera {

    /// Camera position.
    pos  : Vec3F,
    /// Front direction.
    front: Vec3F,
    /// Up direction.
    up   : Vec3F,
    /// right direction.
    right: Vec3F,

    world_up: Vec3F,

    yaw  : f32,
    pitch: f32,

    // camera options
    move_speed: f32,
    _mouse_sensitivity: f32,
    _wheel_sensitivity: f32,

    zoom: f32,
    near: f32,
    far : f32,
    screen_aspect: f32,

    /// Vulkan assumes a viewport origin at the top-left by default.
    /// This leads to the clip space having its +Y axis pointing downwards, contrary to OpenGL's behaviour.
    /// Set `flip_vertically` to true to adapt this change for vulkan(default is true).
    ///
    /// see http://forum.lwjgl.org/index.php?topic=6167.0 for detail.
    flip_vertically: bool,
}

impl FlightCamera {

    pub fn new() -> FlightCameraBuilder {
        FlightCameraBuilder::default()
    }

    pub fn set_move_speed(&mut self, speed: f32) {
        self.move_speed = speed;
    }

    pub fn current_position(&self) -> Vec3F {
        self.pos.clone()
    }

    /// Generate a new view matrix based on camera status.
    pub fn view_matrix(&self) -> Mat4F {

        Mat4F::look_at_rh(self.pos, self.pos + self.front, self.up)
    }

    /// Generate a new projection matrix based on camera status.
    pub fn proj_matrix(&self) -> Mat4F {

        Mat4F::perspective_rh_zo(self.zoom, self.screen_aspect, self.near, self.far)
    }

    pub fn reset_screen_dimension(&mut self, width: u32, height: u32) {
        self.screen_aspect = (width as f32) / (height as f32);
    }

    pub fn flip_vertically(&mut self) {
        self.flip_vertically = !self.flip_vertically;
    }

    pub fn receive_input(&mut self, inputer: &EventController, delta_time: f32) {

        // keyboard
        let velocity = self.move_speed * delta_time;

        if inputer.key.is_key_pressed(VirtualKeyCode::Up) {
            self.pos += self.front * velocity;
        } else if inputer.key.is_key_pressed(VirtualKeyCode::Down) {
            self.pos -= self.front * velocity;
        }

        if inputer.key.is_key_pressed(VirtualKeyCode::Left) {
            self.pos -= self.right * velocity;
        } else if inputer.key.is_key_pressed(VirtualKeyCode::Right) {
            self.pos += self.right * velocity;
        }

        // mouse motion
        if inputer.is_cursor_active() {

            let mouse_motion = inputer.cursor.get_cursor_motion();

            self.yaw += mouse_motion.0;
            self.pitch = num::clamp(self.pitch - mouse_motion.1, -89.0, 89.0);

            // recalculate front, right or up vector only when mouse move.
            self.update_vectors();
        }
    }

    fn update_vectors(&mut self) {
        // calculate the new front vector.
        let front_x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        let front_y = self.pitch.to_radians().sin();
        let front_z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

        // also calculate the right and up vector.
        // Normalize the vectors, because their length gets closer to 0 the move you look up or down which results in slower movement.
        if self.flip_vertically {
            self.front = Vec3F::new(-front_x, front_y, front_z).normalized();
            self.right = Vec3F::cross(self.front, Vec3F::new(self.world_up.x, -self.world_up.y, self.world_up.z));
            self.up    = Vec3F::cross(self.right, self.front);
        } else {
            self.front = Vec3F::new(front_x, front_y, front_z).normalized();
            self.right = Vec3F::cross(self.front, self.world_up);
            self.up    = Vec3F::cross(self.right, self.front);
        }
    }
}

pub struct FlightCameraBuilder {

    pos     : Vec3F,
    world_up: Vec3F,

    yaw  : f32,
    pitch: f32,

    near: f32,
    far : f32,
    screen_aspect: f32,
}

impl Default for FlightCameraBuilder {

    fn default() -> FlightCameraBuilder {
        FlightCameraBuilder {
            pos      : Vec3F::new(0.0, 0.0, 0.0),
            world_up : Vec3F::new(0.0, 1.0, 0.0),
            yaw      : -90.0,
            pitch    : 0.0,
            near     : 0.1,
            far      : 100.0,
            screen_aspect: 1.0,
        }
    }
}

impl FlightCameraBuilder {

    pub fn build(self) -> FlightCamera {

        let mut camera = FlightCamera {
            pos      : self.pos,
            front    : Vec3F::new(0.0, 0.0, -1.0),
            up       : Vec3F::zero(),
            right    : Vec3F::zero(),
            world_up : self.world_up,
            yaw      : self.yaw,
            pitch    : self.pitch,
            near     : self.near,
            far      : self.far,
            screen_aspect: self.screen_aspect,

            move_speed: 2.5,
            _mouse_sensitivity: 1.0,
            _wheel_sensitivity: 1.0,
            zoom: 45.0_f32.to_radians(),

            flip_vertically: true,
        };
        camera.update_vectors();

        camera
    }

    pub fn place_at(mut self, position: Vec3F) -> FlightCameraBuilder {
        self.pos = position; self
    }

    pub fn world_up(mut self, up: Vec3F) -> FlightCameraBuilder {
        self.world_up = up; self
    }

    pub fn yaw(mut self, yaw: f32) -> FlightCameraBuilder {
        self.yaw = yaw; self
    }

    pub fn pitch(mut self, pitch: f32) -> FlightCameraBuilder {
        self.pitch = pitch; self
    }

    pub fn view_distance(mut self, near: f32, far: f32) -> FlightCameraBuilder {
        self.near = near;
        self.far = far; self
    }

    pub fn screen_aspect_ratio(mut self, ratio: f32) -> FlightCameraBuilder {
        self.screen_aspect = ratio; self
    }
}

