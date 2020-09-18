use arduino_uno::hal::port::{
    mode,
    Pin,
};

struct Sensor {
    ddr: arduino_uno::DDR,
    p: Pin<mode::Input<mode::Floating>>,
}

impl Sensor {
    fn read(self) {
        self.p.into_output(&mut ddr);
    }
}
