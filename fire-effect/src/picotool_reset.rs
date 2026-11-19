// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The reset interface through which picotool reboots a running Pico into its USB bootloader.
//!
//! The firmware offers a vendor-specific interface with [`INTERFACE_CLASS`],
//! [`INTERFACE_SUBCLASS`] and [`INTERFACE_PROTOCOL`] and no endpoints. A host sends class requests
//! to it on the control endpoint, addressed to the interface; [`RebootToBootloader::decode`] turns
//! such a request into the action the firmware takes.

/// Interface class of the reset interface: vendor specific.
pub const INTERFACE_CLASS: u8 = 0xff;

/// Interface subclass of the reset interface.
pub const INTERFACE_SUBCLASS: u8 = 0x00;

/// Interface protocol of the reset interface.
pub const INTERFACE_PROTOCOL: u8 = 0x01;

/// `bRequest` of the request that reboots into the USB bootloader.
const REQUEST_BOOTSEL: u8 = 0x01;

/// Bits of `wValue` that switch interfaces of the bootloader off: bit 0 its mass storage, bit 1
/// its PICOBOOT interface.
const DISABLE_INTERFACE_BITS: u16 = 0b11;

/// A request to reboot into the USB bootloader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RebootToBootloader {
    /// Interfaces of the bootloader to switch off, a subset of [`DISABLE_INTERFACE_BITS`].
    disable_interfaces: u16,
}

impl RebootToBootloader {
    /// Decodes a class request that a host sent to the reset interface, from its `bRequest` and
    /// `wValue`.
    ///
    /// Returns `None` for every request other than the reboot into the bootloader; the firmware
    /// rejects those. Of `wValue`, only the two bits that switch interfaces of the bootloader off
    /// are kept. Every other bit is ignored, among them an activity LED a host may name, so the
    /// boot ROM receives no flag beyond these two.
    #[must_use]
    pub fn decode(request: u8, value: u16) -> Option<Self> {
        (request == REQUEST_BOOTSEL).then_some(Self {
            disable_interfaces: value & DISABLE_INTERFACE_BITS,
        })
    }

    /// Returns the interfaces of the bootloader to switch off, as the boot ROM takes them: bit 0
    /// its mass storage, bit 1 its PICOBOOT interface; 0 keeps both.
    #[must_use]
    pub fn disable_interface_mask(self) -> u32 {
        u32::from(self.disable_interfaces)
    }
}

#[cfg(test)]
mod tests {
    //! Decoding the requests of the reset interface.

    use super::{REQUEST_BOOTSEL, RebootToBootloader};

    /// Bit of `wValue` that switches the mass storage of the bootloader off.
    const DISABLE_MASS_STORAGE: u16 = 1 << 0;

    /// Bit of `wValue` that switches the PICOBOOT interface of the bootloader off.
    const DISABLE_PICOBOOT: u16 = 1 << 1;

    /// Bit of `wValue` by which a host names an activity LED for the bootloader.
    const ACTIVITY_LED: u16 = 1 << 8;

    /// Position in `wValue` of the GPIO number of the activity LED.
    const ACTIVITY_PIN_SHIFT: u32 = 9;

    /// A GPIO a host could name as the activity LED.
    const ACTIVITY_PIN: u16 = 25;

    /// `bRequest` of the reboot into the flashed program, which the firmware does not offer.
    const REQUEST_FLASH: u8 = 0x02;

    /// Returns the mask the boot ROM receives for a reboot request with `value`.
    fn mask_for(value: u16) -> u32 {
        RebootToBootloader::decode(REQUEST_BOOTSEL, value)
            .expect("the reboot into the bootloader must be accepted")
            .disable_interface_mask()
    }

    /// A reboot request with `wValue` 0 keeps both interfaces of the bootloader.
    #[test]
    fn plain_request_keeps_both_interfaces() {
        assert_eq!(mask_for(0), 0, "wValue 0 must keep both interfaces");
    }

    /// Each disable bit reaches the boot ROM at its own position, and both together.
    #[test]
    fn disable_bits_reach_the_boot_rom() {
        assert_eq!(
            mask_for(DISABLE_MASS_STORAGE),
            1,
            "bit 0 must switch the mass storage off"
        );
        assert_eq!(
            mask_for(DISABLE_PICOBOOT),
            2,
            "bit 1 must switch PICOBOOT off"
        );
        assert_eq!(
            mask_for(DISABLE_MASS_STORAGE | DISABLE_PICOBOOT),
            3,
            "both bits must switch both interfaces off"
        );
    }

    /// An activity LED named in `wValue` does not reach the boot ROM.
    #[test]
    fn activity_led_is_ignored() {
        let value = ACTIVITY_LED | (ACTIVITY_PIN << ACTIVITY_PIN_SHIFT);
        assert_eq!(mask_for(value), 0, "an activity LED must be ignored");
        assert_eq!(
            mask_for(value | DISABLE_MASS_STORAGE),
            1,
            "an activity LED must not hide the disable bits"
        );
    }

    /// Every bit of `wValue` beyond the two disable bits is ignored.
    #[test]
    fn other_value_bits_are_ignored() {
        assert_eq!(
            mask_for(!(DISABLE_MASS_STORAGE | DISABLE_PICOBOOT)),
            0,
            "bits 2 to 15 must not reach the boot ROM"
        );
        assert_eq!(
            mask_for(u16::MAX),
            3,
            "all bits set must give exactly both disable bits"
        );
    }

    /// Every request other than the reboot into the bootloader is rejected, the reboot into the
    /// flashed program among them.
    #[test]
    fn other_requests_are_rejected() {
        assert_eq!(
            RebootToBootloader::decode(REQUEST_FLASH, 0),
            None,
            "the reboot into the flashed program is not offered"
        );
        for request in (0..=u8::MAX).filter(|&request| request != REQUEST_BOOTSEL) {
            for value in [0, u16::MAX] {
                assert_eq!(
                    RebootToBootloader::decode(request, value),
                    None,
                    "request {request:#04x} with wValue {value:#06x} must be rejected"
                );
            }
        }
    }
}
