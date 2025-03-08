#![feature(abi_avr_interrupt, type_alias_impl_trait)]
#![feature(trait_alias)]
#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use core::{cell::Cell, borrow::Borrow};
use avr_tc1_embassy_time::{define_interrupt, init_system_time};
use embassy_executor::Spawner;
use arduino_hal::{
    default_serial, pac::USART0, pins, Peripherals,
    simple_pwm::{IntoPwmPin, Timer0Pwm, Timer2Pwm, Prescaler},
    port,
};
use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use avr_device;
use embassy_sync::{
    blocking_mutex::{CriticalSectionMutex, raw::NoopRawMutex},
    channel::{self, Channel},
    watch::{Sender, Watch}, signal::Signal,
};
use embassy_time::Timer;
use panic_halt as _;

// Not used in this implementation, but provided as context
enum Mode {
    On,
    Off,
}

define_interrupt!(atmega328p);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize peripherals and system time
    let mut dp = Peripherals::take().unwrap();
    init_system_time(&mut dp.TC1);
    unsafe { avr_device::interrupt::enable() };

    // Setup PWM timers
    let mut timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale64);
    let mut timer2 = Timer2Pwm::new(dp.TC2, Prescaler::Prescale64);

    let pins = pins!(dp);
    // Choose pins for the RGB LED
    let r_pin = pins.d11.into_output();
    let g_pin = pins.d5.into_output();
    let b_pin = pins.d6.into_output();

    // Convert pins into PWM channels
    let mut r = r_pin.into_pwm(&mut timer2);
    let mut g = g_pin.into_pwm(&mut timer0);
    let mut b = b_pin.into_pwm(&mut timer0);
    r.enable();
    g.enable();
    b.enable();
    
    let mut serial = default_serial!(dp, pins, 57600);

    let mut channel = Signal::<NoopRawMutex, char>::new();
    let channel: &'static Signal<NoopRawMutex, char> = unsafe {
        &*((&channel) as *const _)
    };

    spawner.spawn(read_task(channel)).unwrap();

    let max_duty: u8 = 255;

    loop {
        if channel.signaled() {
            if channel.wait().await == 'f' {
                r.set_duty(255);
                g.set_duty(255);
                b.set_duty(255);
                while channel.wait().await != 'o' {
                    Timer::after_micros(10).await;
                }
            }
        }

        for step in 0..=max_duty {
            if channel.signaled() {
                break;
            }
            r.set_duty(step);
            g.set_duty(max_duty - step);
            b.set_duty(max_duty);
            Timer::after_millis(20).await;
        }

        // from green to blue
        for step in 0..=max_duty {
            if channel.signaled() {
                break;
            }
            g.set_duty(step);
            b.set_duty(max_duty - step);
            Timer::after_millis(20).await;
        }

        for step in 0..=max_duty {
            if channel.signaled() {
                break;
            }
            b.set_duty(step);
            r.set_duty(max_duty - step);
            Timer::after_millis(20).await;
        }
    }
}

#[embassy_executor::task]
async fn read_task(signal: &'static Signal<NoopRawMutex, char>) {
    let usart = unsafe { &*USART0::ptr() };
    loop {
        if usart.ucsr0a.read().rxc0().bit_is_set() {
            let byte = usart.udr0.read().bits() as char;
            signal.signal(byte);
        }
        Timer::after_micros(10).await;
    }
}
