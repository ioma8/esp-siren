#![no_std]
#![no_main]

use core::fmt::{self, Debug, Formatter};

use esp_hal::{
    clock::Clocks,
    delay::Delay,
    gpio::{DriveMode, Level, Output, OutputConfig},
    ledc::{
        LSGlobalClkSource, Ledc, LowSpeed,
        channel::{self, ChannelIFace},
        timer::{self, LSClockSource, TimerIFace},
    },
    main,
    peripherals::GPIO25,
    time::Rate,
};

use esp_backtrace as _;

esp_bootloader_esp_idf::esp_app_desc!();

const HIGH_TONE_HZ: u32 = 900;
const LOW_TONE_HZ: u32 = 600;
const TONE_DURATION_MS: u32 = 450;

struct ToneValue {
    frequency: u32,
    duration: u32,
    led_on: bool,
}

const SIREN_TONES: [ToneValue; 2] = [
    ToneValue {
        frequency: HIGH_TONE_HZ,
        duration: TONE_DURATION_MS,
        led_on: true,
    },
    ToneValue {
        frequency: LOW_TONE_HZ,
        duration: TONE_DURATION_MS,
        led_on: false,
    },
];

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let led_pin = peripherals.GPIO2;
    let mut speaker_pin = peripherals.GPIO25;

    let mut led = Output::new(led_pin, Level::Low, OutputConfig::default());
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut timer = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    let delay = Delay::new();

    loop {
        play_tones(
            &ledc,
            &mut timer,
            speaker_pin.reborrow(),
            &mut led,
            &delay,
            &SIREN_TONES,
        )
        .unwrap();
    }
}

enum BuzzerError {
    Timer(timer::Error),
    Channel(channel::Error),
}

impl Debug for BuzzerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timer(error) => formatter.debug_tuple("Timer").field(error).finish(),
            Self::Channel(error) => formatter.debug_tuple("Channel").field(error).finish(),
        }
    }
}

impl From<timer::Error> for BuzzerError {
    fn from(error: timer::Error) -> Self {
        Self::Timer(error)
    }
}

impl From<channel::Error> for BuzzerError {
    fn from(error: channel::Error) -> Self {
        Self::Channel(error)
    }
}

fn play(
    ledc: &Ledc<'_>,
    timer: &mut timer::Timer<'_, LowSpeed>,
    pin: GPIO25<'_>,
    frequency_hz: u32,
) -> Result<(), BuzzerError> {
    if frequency_hz == 0 {
        return mute(ledc, timer, pin);
    }

    let mut duty_resolution = 0;
    let mut clock_divisor = Clocks::get().apb_clock / Rate::from_hz(frequency_hz);

    while clock_divisor > 1 && duty_resolution < 14 {
        clock_divisor >>= 1;
        duty_resolution += 1;
    }

    timer.configure(timer::config::Config {
        duty: timer::config::Duty::try_from(duty_resolution).unwrap(),
        clock_source: LSClockSource::APBClk,
        frequency: Rate::from_hz(frequency_hz),
    })?;

    let mut channel = ledc.channel(channel::Number::Channel0, pin);
    channel.configure(channel::config::Config {
        timer,
        duty_pct: 50,
        drive_mode: DriveMode::PushPull,
    })?;

    Ok(())
}

fn mute(
    ledc: &Ledc<'_>,
    timer: &timer::Timer<'_, LowSpeed>,
    pin: GPIO25<'_>,
) -> Result<(), BuzzerError> {
    if !timer.is_configured() {
        return Ok(());
    }

    let mut channel = ledc.channel(channel::Number::Channel0, pin);
    channel.configure(channel::config::Config {
        timer,
        duty_pct: 0,
        drive_mode: DriveMode::PushPull,
    })?;

    Ok(())
}

fn play_tones(
    ledc: &Ledc<'_>,
    timer: &mut timer::Timer<'_, LowSpeed>,
    mut pin: GPIO25<'_>,
    led: &mut Output<'_>,
    delay: &Delay,
    tones: &[ToneValue],
) -> Result<(), BuzzerError> {
    for tone in tones {
        play(ledc, timer, pin.reborrow(), tone.frequency)?;
        set_led(led, tone.led_on);
        delay.delay_millis(tone.duration);
        mute(ledc, timer, pin.reborrow())?;
    }

    mute(ledc, timer, pin)?;
    Ok(())
}

fn set_led(led: &mut Output<'_>, on: bool) {
    if on {
        led.set_high();
    } else {
        led.set_low();
    }
}
