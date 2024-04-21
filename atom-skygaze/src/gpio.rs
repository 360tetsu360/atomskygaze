use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub enum GPIOType {
    In,
    Out,
}

pub struct GPIOInterface {
    pub number: u32,
    pub gpio_type: GPIOType,
}

impl GPIOInterface {
    pub fn init(&self) -> std::io::Result<bool> {
        let path_str = format!("/sys/class/gpio/gpio{}", self.number);
        let path = Path::new(&path_str);

        if path.is_dir() {
            return Ok(false);
        }

        Self::echo("/sys/class/gpio/export", &self.number.to_string())?;

        match self.gpio_type {
            GPIOType::In => Self::echo(&format!("{}/direction", path_str), "in")?,
            GPIOType::Out => Self::echo(&format!("{}/direction", path_str), "out")?,
        }

        Ok(true)
    }

    pub fn set_value(&self, value: u32) -> std::io::Result<()> {
        Self::echo(
            &format!("/sys/class/gpio/gpio{}/value", self.number),
            &value.to_string(),
        )
    }

    pub fn active_low(&self) -> std::io::Result<u32> {
        let ret = Self::read(&format!("/sys/class/gpio/gpio{}/active_low", self.number))?;
        Ok(ret.parse().unwrap())
    }

    pub fn value(&self) -> std::io::Result<u32> {
        let ret = Self::read(&format!("/sys/class/gpio/gpio{}/value", self.number))?;
        Ok(ret.parse().unwrap())
    }

    pub fn is_on(&self) -> std::io::Result<bool> {
        let active_low = self.active_low()?;
        Ok(self.value()? != active_low)
    }

    fn read(path: &str) -> std::io::Result<String> {
        let ret = fs::read_to_string(path)?;
        Ok(ret)
    }

    fn echo(path: &str, text: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(text.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}

pub const GPIO_LED_ORANGE: GPIOInterface = GPIOInterface {
    number: 0x26,
    gpio_type: GPIOType::Out,
};

pub const GPIO_LED_BLUE: GPIOInterface = GPIOInterface {
    number: 0x27,
    gpio_type: GPIOType::Out,
};

pub const GPIO_LED_IR: GPIOInterface = GPIOInterface {
    number: 0x2F,
    gpio_type: GPIOType::Out,
};

pub const GPIO_IRCUT_TRIG1: GPIOInterface = GPIOInterface {
    number: 0x34,
    gpio_type: GPIOType::Out,
};

pub const GPIO_IRCUT_TRIG2: GPIOInterface = GPIOInterface {
    number: 0x35,
    gpio_type: GPIOType::Out,
};

pub const GPIO_BUTTON: GPIOInterface = GPIOInterface {
    number: 0x33,
    gpio_type: GPIOType::In,
};

pub fn gpio_init() -> std::io::Result<()> {
    if !GPIO_LED_ORANGE.init()? {
        println!("gpio LED orange already initialized.");
    }

    if !GPIO_LED_BLUE.init()? {
        println!("gpio LED blue already initialized.");
    }

    if !GPIO_LED_IR.init()? {
        println!("gpio LED IR already initialized.");
    }

    if !GPIO_IRCUT_TRIG1.init()? {
        println!("gpio IRCUT trigger2 already initialized.");
    }

    if !GPIO_IRCUT_TRIG2.init()? {
        println!("gpio IRCUT trigger2 already initialized.");
    }

    if !GPIO_BUTTON.init()? {
        println!("gpio button already initialized.");
    }

    Ok(())
}

pub enum IRCUTStatus {
    On,
    Off,
}

pub fn ircut_on() -> std::io::Result<()> {
    GPIO_IRCUT_TRIG2.set_value(1)?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    GPIO_IRCUT_TRIG1.set_value(1)
}

pub fn ircut_off() -> std::io::Result<()> {
    GPIO_IRCUT_TRIG2.set_value(0)?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    GPIO_IRCUT_TRIG1.set_value(0)
}

pub fn irled_on() -> std::io::Result<()> {
    GPIO_LED_IR.set_value(1)
}

#[derive(Copy, Clone, Debug)]
pub enum LEDType {
    Orange,
    Blue,
}

pub fn led_on(led: LEDType) -> std::io::Result<()> {
    match led {
        LEDType::Orange => GPIO_LED_ORANGE.set_value(1),
        LEDType::Blue => GPIO_LED_BLUE.set_value(1),
    }
}

pub fn led_off(led: LEDType) -> std::io::Result<()> {
    match led {
        LEDType::Orange => GPIO_LED_ORANGE.set_value(0),
        LEDType::Blue => GPIO_LED_BLUE.set_value(0),
    }
}
