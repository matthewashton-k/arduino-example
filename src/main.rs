#![feature(abi_avr_interrupt)]
#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use arduino_hal::{pins, Peripherals, simple_pwm::{PwmPinOps, IntoPwmPin, Timer2Pwm, Prescaler, Timer1Pwm}};
use arduino_hal::delay_ms;
use avr_device;
use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Take ownership of the device peripherals
    let mut dp = Peripherals::take().unwrap();

    let mut timer2 = Timer2Pwm::new(dp.TC2, Prescaler::Prescale64);
    let mut timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);

    let pins = pins!(dp);
    let mut r = pins.d11.into_output();
    let mut g = pins.d10.into_output();
    let mut b = pins.d9.into_output();

    let mut r = r.into_pwm(&mut timer2);
    let mut g = g.into_pwm(&mut timer1);
    let mut b = b.into_pwm(&mut timer1);
    r.enable();
    g.enable();
    b.enable();
    
    let max_duty: u8 = 255;

    // lower duty cycle apparantly means its brighter lord knows why
    loop {
        // red to green
        for step in 0..=max_duty {
            let red_intensity   = max_duty - step; // Red fades from 255 to 0.
            let green_intensity = step;            // Green fades from 0 to 255.
            let blue_intensity  = 0;                // Blue remains off.
            r.set_duty(max_duty - red_intensity);
            g.set_duty(max_duty - green_intensity);
            b.set_duty(max_duty - blue_intensity);
            delay_ms(5);
        }

        for step in 0..=max_duty {
            let green_intensity = max_duty - step; // Green fades from 255 to 0.
            let blue_intensity  = step;            // Blue fades from 0 to 255.
            let red_intensity   = 0;                // Red remains off.
            r.set_duty(max_duty - red_intensity);
            g.set_duty(max_duty - green_intensity);
            b.set_duty(max_duty - blue_intensity); 
            delay_ms(5);
        }

        for step in 0..=max_duty {
            let blue_intensity = max_duty - step;  // Blue fades from 255 to 0.
            let red_intensity  = step;             // Red fades from 0 to 255.
            let green_intensity = 0;               // Green remains off.
            r.set_duty(max_duty - red_intensity);
            g.set_duty(max_duty - green_intensity);
            b.set_duty(max_duty - blue_intensity);
            delay_ms(5);
        }
    }
}
