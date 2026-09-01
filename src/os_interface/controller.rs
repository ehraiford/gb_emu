use sdl2::{
    EventPump, GameControllerSubsystem, Sdl,
    controller::{
        Button::{self, A, B, Back, DPadDown, DPadLeft, DPadRight, DPadUp, Start},
        GameController,
    },
};

use crate::os_interface::input::{InputSource, NO_BUTTONS_PRESSED};

pub const SDL_BUTTON_MAPPING: [Button; 8] = [
    DPadRight, // Bit 0: Right
    DPadLeft,  // Bit 1: Left
    DPadUp,    // Bit 2: Up
    DPadDown,  // Bit 3: Down
    A,         // Bit 4: A
    B,         // Bit 5: B
    Back,      // Bit 6: Select
    Start,     // Bit 7: Start
];

pub struct ControllerInput {
    controller_subsystem: GameControllerSubsystem,
    active_controllers: Vec<GameController>,
    known_joystick_count: u32,
}

impl ControllerInput {
    pub fn new(sdl: &Sdl) -> Option<Self> {
        let controller_subsystem = sdl.game_controller().ok()?;
        controller_subsystem.set_event_state(false);
        let known_joystick_count = controller_subsystem.num_joysticks().unwrap_or_default();

        let mut this = Self {
            controller_subsystem,
            active_controllers: Vec::new(),
            known_joystick_count,
        };
        this.scan_for_controllers();

        Some(this)
    }

    fn scan_for_controllers(&mut self) {
        self.known_joystick_count = self.controller_subsystem.num_joysticks().unwrap_or_default();

        self.active_controllers = Vec::new();
        for i in 0..self.known_joystick_count {
            if self.controller_subsystem.is_game_controller(i)
                && let Ok(controller) = self.controller_subsystem.open(i)
            {
                self.active_controllers.push(controller)
            }
        }
    }
}

impl InputSource for ControllerInput {
    fn poll(&mut self, _: &EventPump) -> u8 {
        let joystick_count = self.controller_subsystem.num_joysticks().unwrap_or_default();
        if joystick_count != self.known_joystick_count {
            self.scan_for_controllers();
        }

        self.controller_subsystem.update();

        let mut buttons_pressed = NO_BUTTONS_PRESSED;
        for controller in &self.active_controllers {
            for (bit, button) in SDL_BUTTON_MAPPING.iter().enumerate() {
                if controller.button(*button) {
                    buttons_pressed &= !(1 << bit);
                }
            }
        }
        buttons_pressed
    }
}
