use arduino_uno::prelude::*;
use avr_hal_generic::port::mode::{
    Input,
    PullUp,
};

const MAG_ACC_ADDR: u8 = 0b0011101; // Magnetometer/accelerometer
const MAG_ACC_CTRL0: u8 = 0x1f;
const MAG_STATUS_REG: u8 = 0x07;
const MAG_REG_OUT: u8 = 0x08;

const GYRO_ADDR: u8 = 0b1101011; // Gyroscope

pub struct IMU {
    i2c: arduino_uno::I2cMaster<Input<PullUp>>,
}

impl IMU {
    pub fn new(mut i2c: arduino_uno::I2cMaster<Input<PullUp>>) -> IMU {
        // Accelerometer

        // 0101 -> 50Hz output data rate
        // 0111 -> all axes enabled
        i2c.write(MAG_ACC_ADDR, &[MAG_ACC_CTRL0 + 1, 0b01010111])
            .expect("write failed");

        // Magnetometer

        // 0 -> disable temperature sensor
        // 11 -> high resolution mode
        // 001 -> 6.25Hz output data rate
        // 00 -> no interrupt requests are latched
        i2c.write(MAG_ACC_ADDR, &[MAG_ACC_CTRL0 + 5, 0x64])
            .expect("write failed");
        i2c.write(MAG_ACC_ADDR, &[MAG_ACC_CTRL0 + 6, 0x20])
            .expect("write failed");
        i2c.write(MAG_ACC_ADDR, &[MAG_ACC_CTRL0 + 7, 0x00])
            .expect("write failed");

        IMU { i2c }
    }

    pub fn read_magnetometer(&mut self) -> (u16, u16, u16) {
        self.read_axes_16_bit(MAG_ACC_ADDR, MAG_REG_OUT)
    }

    pub fn is_magnetometer_ready(&mut self) -> bool {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(MAG_ACC_ADDR, &[MAG_STATUS_REG], &mut data)
            .expect("write_read failed");
        (data[0] & 0x08) > 0
    }

    fn read_axes_16_bit(&mut self, addr: u8, reg: u8) -> (u16, u16, u16) {
        let mut data: [u8; 6] = [0; 6];
        self.i2c
            .write_read(addr, &[reg | 0x80], &mut data)
            .expect("write_read failed");
        (
            (data[1] as u16) << 8 | data[0] as u16,
            (data[3] as u16) << 8 | data[2] as u16,
            (data[5] as u16) << 8 | data[4] as u16,
        )
    }
}
