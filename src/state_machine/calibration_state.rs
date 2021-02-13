use crate::{
    avr_async::Waiter,
    state_machine::State,
    uno::{
        eeprom::*,
        motor,
        Uno,
    },
};
use arduino_uno::prelude::*;

pub async fn calibration_future(uno: &mut Uno) -> State {
    // Calibrate the IMU
    uno.blink(3, 500).await;

    uno.motor_controller.set_targets(-1.0, 1.0);
    let (x_min, x_max, y_min, y_max) = uno.imu.get_calibration_vector().await;
    uno.motor_controller.set_targets(0.0, 0.0);

    uno.write_eeprom_u16(IMU_X_MIN_ADDR, x_min as u16).await;
    uno.write_eeprom_u16(IMU_X_MAX_ADDR, x_max as u16).await;
    uno.write_eeprom_u16(IMU_Y_MIN_ADDR, y_min as u16).await;
    uno.write_eeprom_u16(IMU_Y_MAX_ADDR, y_max as u16).await;

    // calibrate the IR sensors -- dark first, then light
    // wait for a button press to signal that the robot is positioned
    // over a dark (light) surface
    uno.blink(3, 500).await;

    uno.pushbutton.wait_for_press().await;
    let [s0_max, s1_max, s2_max, s3_max, s4_max, s5_max] = uno.ir_sensors.calibrate(&mut uno.ddr, true).await;
    uno.write_eeprom_u16(IR_0_MAX_ADDR, s0_max as u16).await;
    uno.write_eeprom_u16(IR_1_MAX_ADDR, s1_max as u16).await;
    uno.write_eeprom_u16(IR_2_MAX_ADDR, s2_max as u16).await;
    uno.write_eeprom_u16(IR_3_MAX_ADDR, s3_max as u16).await;
    uno.write_eeprom_u16(IR_4_MAX_ADDR, s4_max as u16).await;
    uno.write_eeprom_u16(IR_5_MAX_ADDR, s5_max as u16).await;

    uno.blink(3, 500).await;

    uno.pushbutton.wait_for_press().await;
    let [s0_min, s1_min, s2_min, s3_min, s4_min, s5_min] = uno.ir_sensors.calibrate(&mut uno.ddr, false).await;
    uno.write_eeprom_u16(IR_0_MIN_ADDR, s0_min as u16).await;
    uno.write_eeprom_u16(IR_1_MIN_ADDR, s1_min as u16).await;
    uno.write_eeprom_u16(IR_2_MIN_ADDR, s2_min as u16).await;
    uno.write_eeprom_u16(IR_3_MIN_ADDR, s3_min as u16).await;
    uno.write_eeprom_u16(IR_4_MIN_ADDR, s4_min as u16).await;
    uno.write_eeprom_u16(IR_5_MIN_ADDR, s5_min as u16).await;

    uno.blink(3, 500).await;

    State::Initialization
}
