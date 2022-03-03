use common::serde_derive::{Deserialize,Serialize};
use rppal::gpio::Gpio as rppal_gpio;
use rppal::gpio::OutputPin;

pub struct Gpio {
    led_pin: OutputPin,
    pump_pin: OutputPin
}

#[derive(Debug,Deserialize,Serialize)]
#[serde(crate = "common::serde")]
/// List of avaliable actions to perform on gpio
pub enum GpioAction { 
    /// Turn output on
    On, 
    /// Turn output off
    Off, 
    /// Toggle output
    Toggle, 
    /// Pulse output on for n seconds
    Pulse(u64)
}

#[derive(Debug,Deserialize,Serialize,Clone,Copy)]
#[serde(crate = "common::serde")]
pub enum GpioOutput { Led, Pump }

impl Gpio {
    pub fn new() -> Self { 
        let dev = rppal_gpio::new().unwrap(); 
        let led_pin = dev.get(5).unwrap().into_output();
        let pump_pin = dev.get(13).unwrap().into_output();
        Self { led_pin, pump_pin } 
    }

    pub fn set(&mut self, c: Command) {
        println!("Setting pin {:?} {:?}",c.output,c.action);
        let output_ref = match c.output {
            GpioOutput::Led  => &mut (self.led_pin),
            GpioOutput::Pump => &mut (self.pump_pin),
        };
        match c.action {
            GpioAction::On     => output_ref.set_high(),
            GpioAction::Off    => output_ref.set_low(),
            GpioAction::Toggle => output_ref.toggle(),
            GpioAction::Pulse(_seconds) => unreachable!(),
        }   
    }
}

#[derive(Deserialize,Serialize,Debug)]
#[serde(crate = "common::serde")]
pub struct Command {
    pub output: GpioOutput, 
    pub action: GpioAction,
}

#[cfg(test)]
mod test {
    use common::serde_json;
    use super::*;
    use std::thread;
    use std::time::Duration;
    #[ignore]
    #[test]
    fn demo_command_serial() {
        let c = Command { output: GpioOutput::Led , action: GpioAction::On };
        let c_serial = serde_json::json!(c);
        println!("c_serial: {c_serial}");
    }
    
    #[ignore]
    #[test]
    fn led_blink() {
        let mut pin = rppal_gpio::new().unwrap().get(5).unwrap().into_output();
        loop {
            pin.toggle();
            thread::sleep(Duration::from_millis(500));
        }
    }
}