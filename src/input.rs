use libusb::{Context, Device, DeviceHandle, Error};
use std::time::Duration;

pub struct Input<'a> {
    adapter_handles: Vec<DeviceHandle<'a>>,
    empty_inputs:    Vec<ControllerInput>,
    current_inputs:  Vec<ControllerInput>,      // inputs for this frame
    game_inputs:     Vec<Vec<ControllerInput>>, // game past and (potentially) future inputs, frame 0 has index 2
    prev_start:      bool,
}

// In/Out is from perspective of computer
// Out means: computer->adapter
// In means:  adapter->computer

impl<'a> Input<'a> {
    pub fn new(context: &'a mut  Context) -> Input<'a> {
        let mut adapter_handles: Vec<DeviceHandle> = Vec::new();
        let devices = context.devices();
        for mut device in devices.unwrap().iter() {
            if let Ok(device_desc) = device.device_descriptor() {
                if device_desc.vendor_id() == 0x057E && device_desc.product_id() == 0x0337 {
                    match device.open() {
                        Ok(mut handle) => {
                            if let Ok(true) = handle.kernel_driver_active(0) {
                                handle.detach_kernel_driver(0).unwrap();
                            }
                            match handle.claim_interface(0) {
                                Ok(_) => {
                                    // Tell adapter to start reading
                                    let payload = [0x13];
                                    if let Ok(_) = handle.write_interrupt(0x2, &payload, Duration::new(1, 0)) {
                                        adapter_handles.push(handle);
                                        println!("GC adapter: Setup complete");
                                    }
                                },
                                Err(e) => { println!("GC adapter: Failed to claim interface: {}", e) }
                            }
                        }
                        Err(e) => {
                            Input::handle_open_error(e);
                        }
                    }
                }
            }
        }

        let mut empty_inputs: Vec<ControllerInput> = vec!();
        for _ in &adapter_handles {
            for _ in 0..4 {
                empty_inputs.push(ControllerInput::empty());
            }
        }

        let input = Input {
            adapter_handles: adapter_handles,
            empty_inputs:    empty_inputs,
            game_inputs:     vec!(),
            current_inputs:  vec!(),
            prev_start:      false,
        };
        input
    }

    fn handle_open_error(e: Error) {
        let access_solution = if cfg!(target_os = "linux") { r#":
    You need to set a udev rule so that the adapter can be accessed.
    To fix this on most Linux distributions, run the following command and then restart your computer.
    echo 'SUBSYSTEM=="usb", ENV{DEVTYPE}=="usb_device", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="0337", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/51-gcadapter.rules"#
        } else { "" };

        match e {
            Error::Access => {
                println!("GC adapter: Permissions error{}", access_solution);

            },
            _ => { println!("GC adapter: Failed to open handle: {:?}", e); },
        }
    }

    /// Call this once every frame
    pub fn update(&mut self) {
        let mut inputs: Vec<ControllerInput> = Vec::new();

        for handle in &mut self.adapter_handles {
            read_gc_adapter(handle, &mut inputs);
        }
        read_usb_controllers(&mut inputs);

        self.current_inputs = inputs;
    }

    /// Reset the game input history
    pub fn reset_history(&mut self) {
        self.game_inputs = vec!();
    }

    /// Call this once from the game update logic only 
    /// Throws out all future history that may exist
    pub fn game_update(&mut self, frame: usize) {
        for _ in frame..(self.game_inputs.len()+1) {
            self.game_inputs.pop();
        }

        self.game_inputs.push(self.current_inputs.clone());
    }


    /// Return game inputs at current index into history
    pub fn players(&self, frame: usize) -> Vec<PlayerInput> {
        let mut result_inputs: Vec<PlayerInput> = vec!();
        let frame = frame as i64;
        let inputs      = self.get_inputs(frame);
        let prev_inputs = self.get_inputs(frame-1);

        for (i, input) in inputs.iter().enumerate() {
            let prev_input = &prev_inputs[i];
            if input.plugged_in {
                result_inputs.push(PlayerInput {
                    plugged_in: true,

                    up:    Button { value: input.up,    press: input.up    && !prev_input.up },
                    down:  Button { value: input.down,  press: input.down  && !prev_input.down },
                    right: Button { value: input.right, press: input.right && !prev_input.right },
                    left:  Button { value: input.left,  press: input.left  && !prev_input.left },
                    y:     Button { value: input.y,     press: input.y     && !prev_input.y },
                    x:     Button { value: input.x,     press: input.x     && !prev_input.x },
                    b:     Button { value: input.b,     press: input.b     && !prev_input.b },
                    a:     Button { value: input.a,     press: input.a     && !prev_input.a },
                    l:     Button { value: input.l,     press: input.l     && !prev_input.l },
                    r:     Button { value: input.r,     press: input.r     && !prev_input.r },
                    z:     Button { value: input.z,     press: input.z     && !prev_input.z },
                    start: Button { value: input.start, press: input.start && !prev_input.start },

                    stick_x:   Stick { value: input.stick_x,   diff: input.stick_x   - prev_input.stick_x },
                    stick_y:   Stick { value: input.stick_y,   diff: input.stick_y   - prev_input.stick_y },
                    c_stick_x: Stick { value: input.c_stick_x, diff: input.c_stick_x - prev_input.c_stick_x },
                    c_stick_y: Stick { value: input.c_stick_y, diff: input.c_stick_y - prev_input.c_stick_y },

                    l_trigger:  Trigger { value: input.l_trigger, diff: input.l_trigger - prev_input.l_trigger },
                    r_trigger:  Trigger { value: input.r_trigger, diff: input.r_trigger - prev_input.r_trigger },
                });
            }
            else {
                result_inputs.push(PlayerInput::empty());
            }
        }
        result_inputs
    }

    fn get_inputs(&self, frame: i64) -> &Vec<ControllerInput> {
        // when players(0) is called it requests inputs on frames 0 and -1
        // when players(1) is called it requests inputs on frames 1 and 0
        // empty inputs are returned for 0 and -1 as:
        // *    frame 0 has no inputs
        // *    frame 1 has no diff
        if frame == 0 || frame == -1 {
            &self.empty_inputs
        }
        else {
            let index = frame as usize - 1;
            &self.game_inputs.get(index).unwrap()
        }
    }

    /// Check for start button press
    /// Uses a seperate state from the game inputs
    /// TODO: Maybe this should be extended to include all menu controller interaction? 
    /// TODO: Does not distinguish between start presses from different players, should it?
    pub fn start_pressed(&mut self) -> bool {
        let held = self.start_held();
        let pressed = !self.prev_start && held;
        self.prev_start = held;
        pressed
    }

    fn start_held(&mut self) -> bool {
        for player in &self.current_inputs {
            if player.start {
                return true;
            }
        }
        false
    }

    /// Returns the index to the last frame in history
    pub fn last_frame(&self) -> usize {
        self.game_inputs.len() - 1
    }
}

#[allow(dead_code)]
fn display_endpoints(device: &mut Device) {
    for interface in device.config_descriptor(0).unwrap().interfaces() {
        println!("interface: {}", interface.number());
        for setting in interface.descriptors() {
            for endpoint in setting.endpoint_descriptors() {
                println!("    endpoint - number: {}, address: {}", endpoint.number(), endpoint.address());
            }
        }
    }
}

impl ControllerInput {
    fn empty() -> ControllerInput {
        ControllerInput {
            plugged_in: false,

            up:    false,
            down:  false,
            right: false,
            left:  false,
            y:     false,
            x:     false,
            b:     false,
            a:     false,
            l:     false,
            r:     false,
            z:     false,
            start: false,

            stick_x:   0.0,
            stick_y:   0.0,
            c_stick_x: 0.0,
            c_stick_y: 0.0,
            l_trigger: 0.0,
            r_trigger: 0.0,
        }
    }
}

impl PlayerInput {
    pub fn empty() -> PlayerInput {
        PlayerInput {
            plugged_in: false,

            up:    Button { value: false, press: false },
            down:  Button { value: false, press: false },
            right: Button { value: false, press: false },
            left:  Button { value: false, press: false },
            y:     Button { value: false, press: false },
            x:     Button { value: false, press: false },
            b:     Button { value: false, press: false },
            a:     Button { value: false, press: false },
            l:     Button { value: false, press: false },
            r:     Button { value: false, press: false },
            z:     Button { value: false, press: false },
            start: Button { value: false, press: false },

            stick_x:   Stick { value: 0.0, diff: 0.0 },
            stick_y:   Stick { value: 0.0, diff: 0.0 },
            c_stick_x: Stick { value: 0.0, diff: 0.0 },
            c_stick_y: Stick { value: 0.0, diff: 0.0 },

            l_trigger:  Trigger { value: 0.0, diff: 0.0 },
            r_trigger:  Trigger { value: 0.0, diff: 0.0 },
        }
    }
}

/// Add 4 GC adapter controllers to inputs
fn read_gc_adapter(handle: &mut DeviceHandle, inputs: &mut Vec<ControllerInput>) {
    let mut data: [u8; 37] = [0; 37];
    handle.read_interrupt(0x81, &mut data, Duration::new(1, 0)).unwrap();

    for port in 0..4 {
        let (stick_x, stick_y)     = stick_filter(data[9*port+4], data[9*port+5]);
        let (c_stick_x, c_stick_y) = stick_filter(data[9*port+6], data[9*port+7]);

        inputs.push(ControllerInput {
            plugged_in: data[9*port+1] == 20 || data[9*port+1] == 16,

            up   : data[9*port+2] & 0b10000000 != 0,
            down : data[9*port+2] & 0b01000000 != 0,
            right: data[9*port+2] & 0b00100000 != 0,
            left : data[9*port+2] & 0b00010000 != 0,
            y    : data[9*port+2] & 0b00001000 != 0,
            x    : data[9*port+2] & 0b00000100 != 0,
            b    : data[9*port+2] & 0b00000010 != 0,
            a    : data[9*port+2] & 0b00000001 != 0,
            l    : data[9*port+3] & 0b00001000 != 0,
            r    : data[9*port+3] & 0b00000100 != 0,
            z    : data[9*port+3] & 0b00000010 != 0,
            start: data[9*port+3] & 0b00000001 != 0,

            l_trigger: trigger_filter(data[9*port+8]),
            r_trigger: trigger_filter(data[9*port+9]),

            stick_x:   stick_x,
            stick_y:   stick_y,
            c_stick_x: c_stick_x,
            c_stick_y: c_stick_y,

        });
    };
}

// TODO: implement
/// Add 4 controllers from usb to inputs
fn read_usb_controllers(inputs: &mut Vec<ControllerInput>) {
    for _ in 0..0 {
        inputs.push(ControllerInput::empty());
    }
}

fn abs_min(a: f32, b: f32) -> f32 {
    if (a >= 0.0 && a > b) || (a <= 0.0 && a < b) {
        b
    } else {
        a
    }
}
fn stick_filter(in_stick_x: u8, in_stick_y: u8) -> (f32, f32) {
    let raw_stick_x = in_stick_x as f32 - 128.0;
    let raw_stick_y = in_stick_y as f32 - 128.0;
    let angle = (raw_stick_y).atan2(raw_stick_x);

    let max = (angle.cos() * 80.0).trunc();
    let stick_x = abs_min(raw_stick_x, max) / 80.0;

    let max = (angle.sin() * 80.0).trunc();
    let stick_y = abs_min(raw_stick_y, max) / 80.0;

    (stick_x, stick_y)
}

fn trigger_filter(trigger: u8) -> f32 {
    let value = (trigger as f32) / 140.0;
    if value > 1.0
    {
        1.0
    }
    else {
        value
    }
}

/// Internal input storage
#[derive(Clone)]
struct ControllerInput {
    pub plugged_in: bool,

    pub a:     bool,
    pub b:     bool,
    pub x:     bool,
    pub y:     bool,
    pub left:  bool,
    pub right: bool,
    pub down:  bool,
    pub up:    bool,
    pub start: bool,
    pub z:     bool,
    pub r:     bool,
    pub l:     bool,

    pub stick_x:   f32,
    pub stick_y:   f32,
    pub c_stick_x: f32,
    pub c_stick_y: f32,
    pub r_trigger: f32,
    pub l_trigger: f32,
}

/// External data access
pub struct PlayerInput {
    pub plugged_in: bool,

    pub a:     Button,
    pub b:     Button,
    pub x:     Button,
    pub y:     Button,
    pub left:  Button,
    pub right: Button,
    pub down:  Button,
    pub up:    Button,
    pub start: Button,
    pub z:     Button,
    pub r:     Button,
    pub l:     Button,

    pub stick_x:   Stick,
    pub stick_y:   Stick,
    pub c_stick_x: Stick,
    pub c_stick_y: Stick,
    pub r_trigger:  Trigger,
    pub l_trigger:  Trigger,
}

pub struct Button {
    pub value: bool, // on
    pub press: bool, // off->on this frame
}

pub struct Stick {
    pub value: f32, // current.value
    pub diff:  f32, // current.value - previous.value
}

pub struct Trigger {
    pub value: f32, // current.value
    pub diff:  f32, // current.value - previous.value
}
