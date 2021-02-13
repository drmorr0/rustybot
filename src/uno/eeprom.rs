use crate::{
    Uno,
    Waiter,
};
use arduino_uno::pac::EEPROM;
use avr_hal_generic::avr_device;
use core::ops::Add;

// The IMU and IR calibration values take two bytes each
pub const IMU_X_MIN_ADDR: u8 = 0;
pub const IMU_X_MAX_ADDR: u8 = 2;
pub const IMU_Y_MIN_ADDR: u8 = 4;
pub const IMU_Y_MAX_ADDR: u8 = 6;
pub const IR_0_MIN_ADDR: u8 = 8;
pub const IR_0_MAX_ADDR: u8 = 10;
pub const IR_1_MIN_ADDR: u8 = 12;
pub const IR_1_MAX_ADDR: u8 = 14;
pub const IR_2_MIN_ADDR: u8 = 16;
pub const IR_2_MAX_ADDR: u8 = 18;
pub const IR_3_MIN_ADDR: u8 = 20;
pub const IR_3_MAX_ADDR: u8 = 22;
pub const IR_4_MIN_ADDR: u8 = 24;
pub const IR_4_MAX_ADDR: u8 = 26;
pub const IR_5_MIN_ADDR: u8 = 28;
pub const IR_5_MAX_ADDR: u8 = 30;
pub const _END_ADDR: u8 = 32;

impl Uno {
    pub async fn read_eeprom_u8(&mut self, addr: u8) -> u8 {
        while self.eeprom.eecr.read().eepe().bit_is_set() {
            Waiter::new(1).await;
        }

        self.eeprom.eear.write(|w| unsafe { w.bits((addr as u8).into()) });
        self.eeprom.eecr.write(|w| w.eere().set_bit());
        self.eeprom.eedr.read().bits()
    }

    pub async fn read_eeprom_u16(&mut self, addr: u8) -> u16 {
        let mut value: u16 = self.read_eeprom_u8(addr).await as u16;
        value |= (self.read_eeprom_u8(addr + 1).await as u16) << 8;
        value
    }

    pub async fn read_eeprom_u32(&mut self, addr: u8) -> u32 {
        let mut value: u32 = self.read_eeprom_u8(addr).await as u32;
        value |= (self.read_eeprom_u8(addr + 1).await as u32) << 8;
        value |= (self.read_eeprom_u8(addr + 2).await as u32) << 16;
        value |= (self.read_eeprom_u8(addr + 3).await as u32) << 24;
        value
    }

    pub async fn write_eeprom_u8(&mut self, addr: u8, value: u8) {
        while self.eeprom.eecr.read().eepe().bit_is_set() {
            Waiter::new(1).await;
        }

        avr_device::interrupt::free(|_| {
            self.eeprom.eear.write(|w| unsafe { w.bits(addr.into()) });
            self.eeprom.eedr.write(|w| unsafe { w.bits(value) });

            // The master write-enable and the write-enable have to be separate instructions
            self.eeprom.eecr.write(|w| w.eempe().set_bit());
            self.eeprom.eecr.write(|w| w.eepe().set_bit());
        });
    }

    pub async fn write_eeprom_u16(&mut self, addr: u8, value: u16) {
        self.write_eeprom_u8(addr, value as u8).await;
        self.write_eeprom_u8(addr + 1, (value >> 8) as u8).await;
    }

    pub async fn write_eeprom_u32(&mut self, addr: u8, value: u16) {
        self.write_eeprom_u8(addr, value as u8).await;
        self.write_eeprom_u8(addr + 1, (value >> 8) as u8).await;
        self.write_eeprom_u8(addr + 2, (value >> 16) as u8).await;
        self.write_eeprom_u8(addr + 3, (value >> 23) as u8).await;
    }
}
