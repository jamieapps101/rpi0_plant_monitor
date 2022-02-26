use serde_derive::{Deserialize,Serialize};
use rppal::gpio::Gpio as rppal_gpio;

pub struct Gpio {
    dev: rppal_gpio
}

#[derive(Debug,Deserialize,Serialize)]
pub enum GpioState { High, Low }

impl Gpio {
    pub fn new() -> Self { 
        Self { dev: rppal_gpio::new().unwrap() } 
    }

    pub fn set(&self, c: Command) {
        println!("Setting pin {} {:?}",c.gpio,c.state);
        let pin_inst = self.dev.get(c.gpio).unwrap();
        match c.state {
            GpioState::High => pin_inst.into_output().set_high(),
            GpioState::Low  => pin_inst.into_output().set_low(),
        }
        
    }
}

#[derive(Deserialize,Serialize)]
pub struct Command {
    gpio: u8, 
    state: GpioState,
}

#[cfg(test)]
mod test {
    use serde_json;
    use super::*;
    #[ignore]
    #[test]
    fn demo_command_serial() {
        let c = Command { gpio: 1, state: GpioState:: High};
        let c_serial = serde_json::json!(c);
        println!("c_serial: {c_serial}");
    }
}