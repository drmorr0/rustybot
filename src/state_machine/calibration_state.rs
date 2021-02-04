use crate::{
    avr_async::Waiter,
    state_machine::State,
    uno::{
        eeprom::{
            IMU_X_MAX_ADDR,
            IMU_X_MIN_ADDR,
            IMU_Y_MAX_ADDR,
            IMU_Y_MIN_ADDR,
        },
        motor,
        Uno,
    },
};
use arduino_uno::prelude::*;

pub async fn calibration_future(uno: &mut Uno) -> State {
    uno.led.toggle().void_unwrap();

    // Calibrate the IMU
    uno.motor_controller.set_targets(-1.0, 1.0);
    let (x_min, x_max, y_min, y_max) = uno.imu.get_calibration_vector().await;
    uno.motor_controller.set_targets(0.0, 0.0);

    uno.write_eeprom_u16(IMU_X_MIN_ADDR, x_min as u16).await;
    uno.write_eeprom_u16(IMU_X_MAX_ADDR, x_max as u16).await;
    uno.write_eeprom_u16(IMU_Y_MIN_ADDR, y_min as u16).await;
    uno.write_eeprom_u16(IMU_Y_MAX_ADDR, y_max as u16).await;

    // TODO calibrate the IR sensors

    uno.led.toggle().void_unwrap();
    State::Initialization
}
