use crate::Waiter;
use arduino_uno::{
    pac::EEPROM,
    prelude::*,
};
use avr_hal_generic::port::mode::{
    Input,
    PullUp,
};
use micromath::F32Ext;

const PI: f32 = 3.1415926;
const MAG_ACC_ADDR: u8 = 0b0011101; // Magnetometer/accelerometer
const MAG_ACC_CTRL0: u8 = 0x1f;
const MAG_STATUS_REG: u8 = 0x07;
const MAG_REG_OUT: u8 = 0x08;

const GYRO_ADDR: u8 = 0b1101011; // Gyroscope

const TOTAL_CALIBRATION_SAMPLES: u32 = 100;
const TIME_BETWEEN_SAMPLES_MS: u32 = 50;
const SMOOTHING_ITERS: u8 = 10;

pub struct IMU {
    i2c: arduino_uno::I2cMaster<Input<PullUp>>,
    x_min: f32,
    x_range: f32,
    y_min: f32,
    y_range: f32,
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
        // 100 -> 50Hz output data rate
        // 00 -> no interrupt requests are latched
        i2c.write(MAG_ACC_ADDR, &[MAG_ACC_CTRL0 + 5, 0x64])
            .expect("write failed");

        // 0x20 = +/- 4 gauss range
        i2c.write(MAG_ACC_ADDR, &[MAG_ACC_CTRL0 + 6, 0x20])
            .expect("write failed");

        // 0x00 = continuous-conversion mode (constantly taking readings)
        i2c.write(MAG_ACC_ADDR, &[MAG_ACC_CTRL0 + 7, 0x00])
            .expect("write failed");

        IMU {
            i2c,
            x_min: 0.0,
            x_range: 0.0,
            y_min: 0.0,
            y_range: 0.0,
        }
    }

    pub async fn get_calibration_vector(&mut self) -> (i16, i16, i16, i16) {
        let (mut x_min, mut x_max, mut y_min, mut y_max) = (i16::MAX, 0i16, i16::MAX, 0i16);
        for _ in 0..TOTAL_CALIBRATION_SAMPLES {
            let (x, y, _) = self.read_magnetometer();
            if x < x_min {
                x_min = x
            } else if x > x_max {
                x_max = x
            }
            if y < y_min {
                y_min = y
            } else if y > y_max {
                y_max = y
            }
            Waiter::new(TIME_BETWEEN_SAMPLES_MS).await;
        }

        (x_min, x_max, y_min, y_max)
    }

    pub fn set_calibration_vector(&mut self, x_min: i16, x_max: i16, y_min: i16, y_max: i16) {
        self.x_min = x_min as f32;
        self.x_range = (x_max - x_min) as f32;
        self.y_min = y_min as f32;
        self.y_range = (y_max - y_min) as f32;
    }

    pub fn get_current_heading_degrees(&mut self) -> f32 {
        let (mut avg_x, mut avg_y) = (0.0, 0.0);
        for _ in 0..SMOOTHING_ITERS {
            let (x, y, _) = self.read_axes_16_bit(MAG_ACC_ADDR, MAG_REG_OUT);
            avg_x += x as f32;
            avg_y += y as f32;
        }
        avg_x /= SMOOTHING_ITERS as f32;
        avg_y /= SMOOTHING_ITERS as f32;
        self.compute_heading_degrees(avg_x, avg_y)
    }

    pub fn read_magnetometer(&mut self) -> (i16, i16, i16) {
        self.read_axes_16_bit(MAG_ACC_ADDR, MAG_REG_OUT)
    }

    pub fn is_magnetometer_ready(&mut self) -> bool {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(MAG_ACC_ADDR, &[MAG_STATUS_REG], &mut data)
            .expect("write_read failed");
        (data[0] & 0x08) > 0
    }

    fn read_axes_16_bit(&mut self, addr: u8, reg: u8) -> (i16, i16, i16) {
        let mut data: [u8; 6] = [0; 6];
        self.i2c
            .write_read(addr, &[reg | 0x80], &mut data)
            .expect("write_read failed");
        (
            ((data[1] as u16) << 8 | data[0] as u16) as i16,
            ((data[3] as u16) << 8 | data[2] as u16) as i16,
            ((data[5] as u16) << 8 | data[4] as u16) as i16,
        )
    }

    fn compute_heading_degrees(&mut self, x: f32, y: f32) -> f32 {
        let (x_scaled, y_scaled) = (
            2.0 * (x - self.x_min) / self.x_range - 1.0,
            2.0 * (y - self.y_min) / self.y_range - 1.0,
        );

        let mut angle = x_scaled.atan2(y_scaled) * 180.0 / PI;
        if angle < 0.0 {
            angle += 360.0;
        }
        angle
    }
}
