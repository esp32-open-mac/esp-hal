//! USB OTG full-speed peripheral

pub use esp_synopsys_usb_otg::UsbBus;
use esp_synopsys_usb_otg::UsbPeripheral;

use crate::{
    peripherals,
    system::{Peripheral, PeripheralClockControl},
    types::InputSignal,
};

#[doc(hidden)]
pub trait UsbSel {}

#[doc(hidden)]
pub trait UsbDp {}

#[doc(hidden)]
pub trait UsbDm {}

pub struct USB<S, P, M>
where
    S: UsbSel + Send + Sync,
    P: UsbDp + Send + Sync,
    M: UsbDm + Send + Sync,
{
    _usb0: peripherals::USB0,
    _usb_sel: S,
    _usb_dp: P,
    _usb_dm: M,
}

impl<S, P, M> USB<S, P, M>
where
    S: UsbSel + Send + Sync,
    P: UsbDp + Send + Sync,
    M: UsbDm + Send + Sync,
{
    pub fn new(
        usb0: peripherals::USB0,
        usb_sel: S,
        usb_dp: P,
        usb_dm: M,
        peripheral_clock_control: &mut PeripheralClockControl,
    ) -> Self {
        peripheral_clock_control.enable(Peripheral::Usb);
        Self {
            _usb0: usb0,
            _usb_sel: usb_sel,
            _usb_dp: usb_dp,
            _usb_dm: usb_dm,
        }
    }
}

unsafe impl<S, P, M> Sync for USB<S, P, M>
where
    S: UsbSel + Send + Sync,
    P: UsbDp + Send + Sync,
    M: UsbDm + Send + Sync,
{
}

unsafe impl<S, P, M> UsbPeripheral for USB<S, P, M>
where
    S: UsbSel + Send + Sync,
    P: UsbDp + Send + Sync,
    M: UsbDm + Send + Sync,
{
    const REGISTERS: *const () = peripherals::USB0::ptr() as *const ();

    const HIGH_SPEED: bool = false;
    const FIFO_DEPTH_WORDS: usize = 256;
    const ENDPOINT_COUNT: usize = 5;

    fn enable() {
        unsafe {
            let usb_wrap = &*peripherals::USB_WRAP::PTR;
            usb_wrap.otg_conf.modify(|_, w| {
                w.usb_pad_enable()
                    .set_bit()
                    .phy_sel()
                    .clear_bit()
                    .clk_en()
                    .set_bit()
                    .ahb_clk_force_on()
                    .set_bit()
                    .phy_clk_force_on()
                    .set_bit()
            });

            #[cfg(esp32s3)]
            {
                let rtc = &*peripherals::RTC_CNTL::PTR;
                rtc.usb_conf
                    .modify(|_, w| w.sw_hw_usb_phy_sel().set_bit().sw_usb_phy_sel().set_bit());
            }

            crate::gpio::connect_high_to_peripheral(InputSignal::USB_OTG_IDDIG); // connected connector is mini-B side
            crate::gpio::connect_high_to_peripheral(InputSignal::USB_SRP_BVALID); // HIGH to force USB device mode
            crate::gpio::connect_high_to_peripheral(InputSignal::USB_OTG_VBUSVALID); // receiving a valid Vbus from device
            crate::gpio::connect_low_to_peripheral(InputSignal::USB_OTG_AVALID);

            usb_wrap.otg_conf.modify(|_, w| {
                w.pad_pull_override()
                    .set_bit()
                    .dp_pullup()
                    .set_bit()
                    .dp_pulldown()
                    .clear_bit()
                    .dm_pullup()
                    .clear_bit()
                    .dm_pulldown()
                    .clear_bit()
            });
        }
    }

    fn ahb_frequency_hz(&self) -> u32 {
        // unused
        80_000_000
    }
}