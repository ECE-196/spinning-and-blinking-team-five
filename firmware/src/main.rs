#![no_std]
#![no_main]

use fugit::RateExtU32;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::{
    clock::Clocks, gpio::{Io, Level, Output}, mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, McPwm, PeripheralClockConfig}, timer::timg::TimerGroup
};
use esp_println::println;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led = Output::new(io.pins.gpio17, Level::Low);
    // Spin one way
    // let mut motor_hi = Output::new(io.pins.gpio13, Level::High /* set default pin configuration to HIGH */);

    // Spin other way
    // let mut motor_lo = Output::new(io.pins.gpio14, Level::Low);

    let motor_hi_pin = io.pins.gpio13;
    let motor_lo_pin = io.pins.gpio14;

    let clocks = Clocks::get();

    println!("src clock: {}", clocks.crypto_pwm_clock);

    let clock_cfg = PeripheralClockConfig::with_frequency(32.MHz()).unwrap();
    let mut mcpwm = McPwm::new(peripherals.MCPWM0, clock_cfg);
    mcpwm.operator0.set_timer(&mcpwm.timer0);

    let (mut motor_hi, mut motor_lo) = mcpwm.operator0.with_pins(
    motor_hi_pin,
    PwmPinConfig::UP_ACTIVE_HIGH,
    motor_lo_pin,
    PwmPinConfig::UP_ACTIVE_HIGH,
    );

    let timer_clock_cfg = clock_cfg
    .timer_clock_with_frequency(u8::MAX as u16, PwmWorkingMode::Increase, 20.kHz())
    .unwrap();

    mcpwm.timer0.start(timer_clock_cfg);

    motor_lo.set_timestamp(0);
    let mut torque_step: i16 = 25;
    let mut torque: i16 = 150;
    let mut is_right: bool = false;

    loop {
        println!("{}", torque);
        led.toggle();
        Timer::after_millis(1_000).await;
        if torque >= 255 {
            torque_step *= -1;
        } else if torque <150{
            torque_step *= -1;
            is_right = !is_right;
        }
        if is_right {
            motor_lo.set_timestamp(0);
            motor_hi.set_timestamp(torque as u16);
        } else {
            motor_hi.set_timestamp(0);
            motor_lo.set_timestamp((torque) as u16);
        }
        torque += torque_step;
    }
}
