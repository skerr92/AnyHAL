//! Host-side fakes for executable HAL specifications.

use crate::hal::{
    adc::OneShotAdc,
    capture::{PulseCapture, PulseMeasurement},
    dac::AnalogOutput,
    delay::DelayMs,
    gpio::{AlternateConfig, InputPin, OutputPin, Pin},
    i2c::I2cBus,
    pwm::PwmOutput,
    serial::SerialConsole,
    spi::SpiBus,
    Error, Result,
};

#[derive(Debug)]
pub struct MockPulseCapture {
    tick_hz: u32,
    pending: Option<PulseMeasurement>,
}

impl MockPulseCapture {
    pub const fn new(tick_hz: u32) -> Self {
        Self {
            tick_hz,
            pending: None,
        }
    }

    pub fn inject(&mut self, measurement: PulseMeasurement) {
        self.pending = Some(measurement);
    }
}

impl PulseCapture for MockPulseCapture {
    fn tick_hz(&self) -> u32 {
        self.tick_hz
    }

    fn try_measure(&mut self) -> Result<Option<PulseMeasurement>> {
        Ok(self.pending.take())
    }
}

#[derive(Debug)]
pub struct MockAdc {
    sample: u16,
    resolution_bits: u8,
    reference_mv: u32,
}

impl MockAdc {
    pub const fn new(sample: u16, resolution_bits: u8, reference_mv: u32) -> Self {
        Self {
            sample,
            resolution_bits,
            reference_mv,
        }
    }

    pub fn set_sample(&mut self, sample: u16) {
        self.sample = sample;
    }
}

impl OneShotAdc for MockAdc {
    fn resolution_bits(&self) -> u8 {
        self.resolution_bits
    }

    fn reference_mv(&self) -> u32 {
        self.reference_mv
    }

    fn read(&mut self) -> Result<u16> {
        Ok(self.sample)
    }
}

#[derive(Debug)]
pub struct MockDac {
    value: u16,
    resolution_bits: u8,
    reference_mv: u32,
}

impl MockDac {
    pub const fn new(resolution_bits: u8, reference_mv: u32) -> Self {
        Self {
            value: 0,
            resolution_bits,
            reference_mv,
        }
    }
}

impl AnalogOutput for MockDac {
    fn resolution_bits(&self) -> u8 {
        self.resolution_bits
    }

    fn reference_mv(&self) -> u32 {
        self.reference_mv
    }

    fn value(&self) -> u16 {
        self.value
    }

    fn set_value(&mut self, value: u16) -> Result<()> {
        let maximum = ((1_u32 << self.resolution_bits) - 1) as u16;
        if value > maximum {
            return Err(Error::InvalidArgument);
        }
        self.value = value;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MockInputPin {
    high: bool,
}

impl MockInputPin {
    pub const fn new(initially_high: bool) -> Self {
        Self {
            high: initially_high,
        }
    }

    pub fn set_state(&mut self, high: bool) {
        self.high = high;
    }
}

impl InputPin for MockInputPin {
    fn is_high(&self) -> bool {
        self.high
    }
}

#[derive(Debug)]
pub struct MockAlternatePin {
    pin: Pin,
    config: AlternateConfig,
}

impl MockAlternatePin {
    pub const fn new(pin: Pin, config: AlternateConfig) -> Self {
        Self { pin, config }
    }

    pub const fn pin(&self) -> Pin {
        self.pin
    }

    pub const fn config(&self) -> AlternateConfig {
        self.config
    }
}

#[derive(Debug, Default)]
pub struct MockOutputPin {
    high: bool,
    transitions: u32,
}

impl MockOutputPin {
    pub const fn new(initially_high: bool) -> Self {
        Self {
            high: initially_high,
            transitions: 0,
        }
    }

    pub const fn transitions(&self) -> u32 {
        self.transitions
    }
}

impl OutputPin for MockOutputPin {
    fn set_low(&mut self) -> Result<()> {
        if self.high {
            self.transitions += 1;
        }
        self.high = false;
        Ok(())
    }

    fn set_high(&mut self) -> Result<()> {
        if !self.high {
            self.transitions += 1;
        }
        self.high = true;
        Ok(())
    }

    fn toggle(&mut self) -> Result<()> {
        self.high = !self.high;
        self.transitions += 1;
        Ok(())
    }

    fn is_set_high(&self) -> bool {
        self.high
    }
}

#[derive(Debug, Default)]
pub struct MockDelay {
    elapsed_ms: u64,
}

impl MockDelay {
    pub const fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms
    }
}

impl DelayMs for MockDelay {
    fn delay_ms(&mut self, milliseconds: u32) -> Result<()> {
        self.elapsed_ms += u64::from(milliseconds);
        Ok(())
    }
}

/// Small register-style I2C target for portable driver tests.
#[derive(Debug)]
pub struct MockI2c {
    address: u8,
    registers: [u8; 256],
    cursor: u8,
}

impl MockI2c {
    pub const fn new(address: u8) -> Self {
        Self {
            address,
            registers: [0; 256],
            cursor: 0,
        }
    }

    pub fn registers_mut(&mut self) -> &mut [u8; 256] {
        &mut self.registers
    }

    fn select(&self, address: u8) -> Result<()> {
        if address < 0x80 && address == self.address {
            Ok(())
        } else {
            Err(Error::NoAcknowledge)
        }
    }
}

impl I2cBus for MockI2c {
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<()> {
        self.select(address)?;
        if let Some((&start, data)) = bytes.split_first() {
            self.cursor = start;
            for &byte in data {
                self.registers[usize::from(self.cursor)] = byte;
                self.cursor = self.cursor.wrapping_add(1);
            }
        }
        Ok(())
    }

    fn read(&mut self, address: u8, bytes: &mut [u8]) -> Result<()> {
        self.select(address)?;
        for byte in bytes {
            *byte = self.registers[usize::from(self.cursor)];
            self.cursor = self.cursor.wrapping_add(1);
        }
        Ok(())
    }

    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8]) -> Result<()> {
        self.write(address, write)?;
        self.read(address, read)
    }
}

#[derive(Debug)]
pub struct MockSerialConsole<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> Default for MockSerialConsole<N> {
    fn default() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }
}

impl<const N: usize> MockSerialConsole<N> {
    pub fn output(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl<const N: usize> SerialConsole for MockSerialConsole<N> {
    fn poll(&mut self) -> bool {
        false
    }

    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        let count = bytes.len().min(N - self.len);
        if count == 0 && !bytes.is_empty() {
            return Err(Error::WouldBlock);
        }
        self.bytes[self.len..self.len + count].copy_from_slice(&bytes[..count]);
        self.len += count;
        Ok(count)
    }

    fn read(&mut self, _bytes: &mut [u8]) -> Result<usize> {
        Err(Error::WouldBlock)
    }
}

#[derive(Debug, Default)]
pub struct MockSpiLoopback {
    transfers: u32,
}

impl MockSpiLoopback {
    pub const fn transfers(&self) -> u32 {
        self.transfers
    }
}

impl SpiBus for MockSpiLoopback {
    fn transfer_byte(&mut self, byte: u8) -> Result<u8> {
        self.transfers += 1;
        Ok(byte)
    }
}

#[derive(Debug)]
pub struct MockPwmOutput {
    frequency_hz: u32,
    duty_cycle: u16,
    enabled: bool,
}

impl MockPwmOutput {
    pub const fn new(frequency_hz: u32) -> Self {
        Self {
            frequency_hz,
            duty_cycle: 0,
            enabled: false,
        }
    }
}

impl PwmOutput for MockPwmOutput {
    fn frequency_hz(&self) -> u32 {
        self.frequency_hz
    }

    fn duty_cycle(&self) -> u16 {
        self.duty_cycle
    }

    fn set_duty_cycle(&mut self, duty_cycle: u16) -> Result<()> {
        self.duty_cycle = duty_cycle;
        Ok(())
    }

    fn enable(&mut self) -> Result<()> {
        self.enabled = true;
        Ok(())
    }

    fn disable(&mut self) -> Result<()> {
        self.enabled = false;
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_tracks_state_and_transitions() {
        let mut pin = MockOutputPin::new(false);
        pin.set_high().unwrap();
        pin.set_high().unwrap();
        pin.toggle().unwrap();
        assert!(!pin.is_set_high());
        assert_eq!(pin.transitions(), 2);
    }

    #[test]
    fn input_reports_driven_state() {
        let mut pin = MockInputPin::new(true);
        assert!(pin.is_high());
        assert!(!pin.is_low());
        pin.set_state(false);
        assert!(pin.is_low());
    }

    #[test]
    fn alternate_function_rejects_out_of_range_number() {
        assert!(crate::hal::gpio::AlternateFunction::new(15).is_some());
        assert!(crate::hal::gpio::AlternateFunction::new(16).is_none());
    }

    #[test]
    fn delay_accumulates_elapsed_time() {
        let mut delay = MockDelay::default();
        delay.delay_ms(250).unwrap();
        delay.delay_ms(750).unwrap();
        assert_eq!(delay.elapsed_ms(), 1_000);
    }

    #[test]
    fn i2c_register_target_supports_write_read() {
        let mut bus = MockI2c::new(0x52);
        bus.registers_mut()[0x10] = 0xa5;
        let mut value = [0];
        bus.write_read(0x52, &[0x10], &mut value).unwrap();
        assert_eq!(value, [0xa5]);
        assert_eq!(bus.read(0x53, &mut value), Err(Error::NoAcknowledge));
    }

    #[test]
    fn serial_console_captures_non_blocking_output() {
        let mut console = MockSerialConsole::<8>::default();
        assert_eq!(console.write(b"AnyHAL").unwrap(), 6);
        assert_eq!(console.output(), b"AnyHAL");
        assert_eq!(console.write(b"!!?").unwrap(), 2);
        assert_eq!(console.write(b"x"), Err(Error::WouldBlock));
    }

    #[test]
    fn spi_loopback_transfers_in_place() {
        let mut spi = MockSpiLoopback::default();
        let mut pattern = [0x00, 0x3c, 0xa5, 0xff];
        spi.transfer_in_place(&mut pattern).unwrap();
        assert_eq!(pattern, [0x00, 0x3c, 0xa5, 0xff]);
        assert_eq!(spi.transfers(), 4);
    }

    #[test]
    fn pwm_tracks_normalized_duty_and_enable_state() {
        let mut pwm = MockPwmOutput::new(20_000);
        assert_eq!(pwm.frequency_hz(), 20_000);
        assert_eq!(pwm.duty_cycle(), 0);
        assert!(!pwm.is_enabled());
        pwm.set_duty_cycle(u16::MAX / 2).unwrap();
        pwm.enable().unwrap();
        assert_eq!(pwm.duty_cycle(), u16::MAX / 2);
        assert!(pwm.is_enabled());
        pwm.disable().unwrap();
        assert!(!pwm.is_enabled());
    }

    #[test]
    fn adc_converts_raw_sample_to_millivolts() {
        let mut adc = MockAdc::new(2048, 12, 3300);
        assert_eq!(adc.read().unwrap(), 2048);
        assert_eq!(adc.read_mv().unwrap(), 1650);
        adc.set_sample(4095);
        assert_eq!(adc.read_mv().unwrap(), 3300);
    }

    #[test]
    fn dac_converts_millivolts_and_rejects_out_of_range_codes() {
        let mut dac = MockDac::new(12, 3300);
        dac.set_mv(1650).unwrap();
        assert_eq!(dac.value(), 2047);
        dac.set_mv(5000).unwrap();
        assert_eq!(dac.value(), 4095);
        assert_eq!(dac.set_value(4096), Err(Error::InvalidArgument));
    }

    #[test]
    fn pulse_capture_reports_period_and_high_time() {
        let mut capture = MockPulseCapture::new(750_000);
        capture.inject(PulseMeasurement {
            period_ticks: 15_000,
            high_ticks: 1_125,
        });
        let sample = capture.try_measure().unwrap().unwrap();
        assert_eq!(sample.period_us(capture.tick_hz()), 20_000);
        assert_eq!(sample.high_us(capture.tick_hz()), 1_500);
        assert_eq!(capture.try_measure().unwrap(), None);
    }
}
