use serde_derive::{Deserialize,Serialize};
use rppal::gpio::Gpio as rppal_gpio;
use rppal::gpio::OutputPin;

pub struct Gpio {
    dev: rppal_gpio,
    led_pin: OutputPin
}

#[derive(Debug,Deserialize,Serialize)]
pub enum GpioState { High, Low }

impl Gpio {
    pub fn new() -> Self { 
        let dev = rppal_gpio::new().unwrap(); 
        let led_pin = dev.get(5).unwrap().into_output();
        Self { dev, led_pin } 
    }

    pub fn set(&mut self, c: Command) {
        println!("Setting pin {} {:?}",c.gpio,c.state);
        match c.state {
            GpioState::High => self.led_pin.set_high(),
            GpioState::Low  => self.led_pin.set_low(),
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