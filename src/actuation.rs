
pub struct GPIO {

}

#[derive(Debug)]
pub enum GpioState { High, Low }

impl GPIO {
    pub fn new() -> Self { Self {} }

    pub fn set(gpio: u8, state: GpioState) {
        println!("Setting pin {gpio} {state:?}");
    }
}