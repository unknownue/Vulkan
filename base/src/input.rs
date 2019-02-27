
use smallvec::SmallVec;
use crate::utils::frame::FrameAction;
use crate::utils::fps::FpsCounter;

const SIMULTANEOUS_KEY_COUNT: usize = 12;


pub struct EventController {

    pub key: KeyHeap,
    pub cursor: CursorMotion,
    pub fps_counter: FpsCounter,

    action: FrameAction,
    is_toggle_key: bool,
    is_toggle_cursor: bool,
}

impl Default for EventController {

    fn default() -> EventController {

        EventController {
            key: Default::default(),
            cursor: Default::default(),
            fps_counter: FpsCounter::new(),

            action: FrameAction::Rendering,
            is_toggle_key: false,
            is_toggle_cursor: false,
        }
    }
}

impl EventController {

    pub(crate) fn record_event(&mut self, event: winit::Event) {

        match event {
            | winit::Event::DeviceEvent { event, .. } => {
                match event {
                    | winit::DeviceEvent::MouseMotion { delta } => {
                        self.cursor.record_motion(delta.0, delta.1);
                        self.is_toggle_cursor = true;
                    },
                    | _ => (),
                }
            },
            | winit::Event::WindowEvent { event, .. } => {
                match event {
                    | winit::WindowEvent::KeyboardInput { input, .. } => {
                        if let Some(code) = input.virtual_keycode {
                            match input.state {
                                | winit::ElementState::Pressed  => {
                                    self.key.key_press(code);
                                    self.is_toggle_key = true;
                                },
                                | winit::ElementState::Released => {
                                    self.key.key_release(code);
                                },
                            }
                        }
                    },
                    | winit::WindowEvent::Resized(_) => {

                        // TODO: When window was created, Resized event will be toggled.
                        // self.action = FrameAction::SwapchainRecreate;
                    },
                    | winit::WindowEvent::CloseRequested => {
                        self.action = FrameAction::Terminal;
                    },
                    | _ => (),
                }
            },
            | _ => {},
        }
    }

    pub fn is_key_active(&self) -> bool {
        self.is_toggle_key
    }

    pub fn is_cursor_active(&self) -> bool {
        self.is_toggle_cursor
    }

    pub(crate) fn tick_frame(&mut self) {

        self.fps_counter.tick_frame();
        self.is_toggle_key = false;
        self.is_toggle_cursor = false;
        self.action = FrameAction::Rendering;
    }

    pub(crate) fn current_action(&self) -> FrameAction {
        self.action
    }
}




pub struct KeyHeap {

    keys: SmallVec<[winit::VirtualKeyCode; SIMULTANEOUS_KEY_COUNT]>,
}

impl Default for KeyHeap {

    fn default() -> KeyHeap {
        KeyHeap { keys: SmallVec::new(), }
    }
}

impl KeyHeap {

    fn key_press(&mut self, code: winit::VirtualKeyCode) {

        // if input key has been existed, just ignore it.
        if self.keys.iter().any(|&key_code| key_code == code) {
            return
        }

        // and the key pool has been full, just ignore the input key.
        if self.keys.len() < SIMULTANEOUS_KEY_COUNT {
            self.keys.push(code);
        }
    }

    fn key_release(&mut self, code: winit::VirtualKeyCode) {

        if let Some(index) = self.keys.iter().position(|&key_code| key_code == code) {
            self.keys.swap_remove(index);
        }
    }

    // TODO: implement is_action_just_pressed, is_action_just_released, and is_action_pressed.
    pub fn is_key_pressed(&self, code: winit::VirtualKeyCode) -> bool {

        self.keys.iter().any(|&key_code| key_code == code)
    }
}


pub struct CursorMotion {

    delta_x: f32,
    delta_y: f32,

    scale_factor: f32,
}

impl Default for CursorMotion {

    fn default() -> CursorMotion {

        CursorMotion {
            delta_x: 0.0,
            delta_y: 0.0,
            scale_factor: 1.0,
        }
    }
}

impl CursorMotion {

    fn record_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.delta_x = (delta_x as f32) * self.scale_factor;
        self.delta_y = (delta_y as f32) * self.scale_factor;
    }

    pub fn get_cursor_motion(&self) -> (f32, f32) {
        (self.delta_x, self.delta_y)
    }
}
