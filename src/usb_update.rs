// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The USB update: a USB device with the reset interface of picotool, so a host reboots the Pico
//! into its USB bootloader and flashes it without the BOOTSEL button.
//!
//! The device has a single vendor-specific interface without endpoints and identifies itself with
//! Raspberry Pi's vendor ID and the product ID picotool looks for. Microsoft OS 2.0 descriptors
//! ask Windows to bind its WinUSB driver to the device, so picotool reaches it without a driver
//! installation. Without a host, on a plain USB charger, the device waits for one and does
//! nothing else.

use embassy_futures::join::join;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{Peri, bind_interrupts, rom_data};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embassy_usb::control::{OutResponse, Recipient, Request, RequestType};
use embassy_usb::msos::{
    CompatibleIdFeatureDescriptor, PropertyData, RegistryPropertyFeatureDescriptor, windows_version,
};
use embassy_usb::types::{InterfaceNumber, StringIndex};
use embassy_usb::{Builder, Config, Handler};
use fire_effect::picotool_reset::{
    INTERFACE_CLASS, INTERFACE_PROTOCOL, INTERFACE_SUBCLASS, RebootToBootloader,
};

use crate::PROGRAM_NAME;

bind_interrupts!(
    /// Routes the USB interrupt to the USB driver of embassy-rp.
    struct Irqs {
        USBCTRL_IRQ => InterruptHandler<USB>;
    }
);

/// Raspberry Pi's USB vendor ID; picotool only looks at devices that carry it.
const VENDOR_ID: u16 = 0x2e8a;

/// One of the product IDs picotool looks for: the one a Pico 2 reports while it runs a program.
const PRODUCT_ID: u16 = 0x0009;

/// Device class 0: the interface names its own class.
const CLASS_PER_INTERFACE: u8 = 0x00;

/// The product name the device reports, the program name of `picotool info`.
const PRODUCT: &str = match PROGRAM_NAME.to_str() {
    Ok(name) => name,
    Err(_) => panic!("the program name must be valid UTF-8"),
};

/// Name of the reset interface.
const INTERFACE_NAME: &str = "Reset";

/// Device configuration: one function with one interface, so no interface association
/// descriptor and no composite device.
const CONFIG: Config<'static> = {
    let mut config = Config::new(VENDOR_ID, PRODUCT_ID);
    config.product = Some(PRODUCT);
    config.composite_with_iads = false;
    config.device_class = CLASS_PER_INTERFACE;
    config.device_sub_class = CLASS_PER_INTERFACE;
    config.device_protocol = CLASS_PER_INTERFACE;
    config
};

/// Vendor request code by which Windows reads the Microsoft OS 2.0 descriptor set; any code that
/// no other vendor request of the device uses.
const MSOS_VENDOR_CODE: u8 = 0x20;

/// Compatible ID that makes Windows bind its WinUSB driver to the device.
const WINUSB: &str = "WINUSB";

/// Most bytes of a compatible ID in its Microsoft OS 2.0 descriptor.
const COMPATIBLE_ID_MAX_LEN: usize = 8;

/// Name of the registry property that holds the device interface GUIDs.
const INTERFACE_GUIDS_NAME: &str = "DeviceInterfaceGUIDs";

/// Device interface GUID of the reset interface, chosen at random for this firmware.
const INTERFACE_GUID: &str = "{6126EDE9-DA0B-4DEE-BE0B-7B60FFBF9688}";

/// Bytes of a configuration or interface descriptor.
const STANDARD_DESCRIPTOR_LEN: usize = 9;

/// Bytes of the configuration descriptor: the configuration and the reset interface, which has no
/// endpoints.
const CONFIG_DESCRIPTOR_LEN: usize = 2 * STANDARD_DESCRIPTOR_LEN;

/// Bytes of the BOS descriptor: its header (5), the USB 2.0 extension embassy-usb always adds (7)
/// and the Microsoft OS 2.0 platform capability (28).
const BOS_DESCRIPTOR_LEN: usize = 5 + 7 + 28;

/// Bytes of a terminating NUL in UTF-16.
const UTF16_NUL_LEN: usize = 2;

/// Returns the bytes of the ASCII `text` in UTF-16 with its terminating NUL.
const fn utf16_len(text: &str) -> usize {
    2 * text.len() + UTF16_NUL_LEN
}

/// Bytes of the Microsoft OS 2.0 descriptor set: its header (10), the compatible ID (20) and the
/// registry property, made of its header (10), its name and a `REG_MULTI_SZ` list with the GUID,
/// which ends with a second NUL.
const MSOS_DESCRIPTOR_LEN: usize =
    10 + 20 + 10 + utf16_len(INTERFACE_GUIDS_NAME) + utf16_len(INTERFACE_GUID) + UTF16_NUL_LEN;

/// Bytes of the control buffer: one packet of the control endpoint. The requests of the reset
/// interface carry no data, and a string descriptor has to be shorter than the buffer.
const CONTROL_BUF_LEN: usize = 64;

/// Returns whether embassy-usb can send `text` as a string descriptor: ASCII, and the descriptor
/// (2 bytes of header, 2 per character) shorter than the control buffer.
const fn fits_string_descriptor(text: &str) -> bool {
    text.is_ascii() && 2 + 2 * text.len() < CONTROL_BUF_LEN
}

const _: () = assert!(
    fits_string_descriptor(PRODUCT) && fits_string_descriptor(INTERFACE_NAME),
    "every string of the USB device must fit the control buffer, or reading it panics"
);

const _: () = assert!(
    WINUSB.is_ascii()
        && INTERFACE_GUIDS_NAME.is_ascii()
        && INTERFACE_GUID.is_ascii()
        && WINUSB.len() <= COMPATIBLE_ID_MAX_LEN,
    "the descriptor lengths assume ASCII strings, and a longer compatible ID panics at start-up"
);

/// Time the host gets to complete the status stage of an accepted reboot request before the chip
/// resets, so the transfer ends without an error; the host fetches it within the next few USB
/// frames of 1 ms.
const STATUS_STAGE_GRACE: Duration = Duration::from_millis(10);

/// GPIO mask of the activity LED of the bootloader: none.
const NO_ACTIVITY_LED: u32 = 0;

/// Accepted reboot requests, from the reset interface to [`reboot_on_request`].
type Requests = Signal<NoopRawMutex, RebootToBootloader>;

/// The reset interface: accepts a reboot request of the host and passes it on.
struct ResetInterface<'a> {
    /// Number of the reset interface; requests to any other interface are not for it.
    number: InterfaceNumber,
    /// Index of [`INTERFACE_NAME`].
    name: StringIndex,
    /// Where an accepted reboot request goes.
    requests: &'a Requests,
}

impl<'a> ResetInterface<'a> {
    /// Adds the reset interface to `builder` and returns its handler, which passes accepted
    /// requests to `requests`.
    fn add_to<'d>(builder: &mut Builder<'d, Driver<'d, USB>>, requests: &'a Requests) -> Self {
        let mut function =
            builder.function(INTERFACE_CLASS, INTERFACE_SUBCLASS, INTERFACE_PROTOCOL);
        let mut interface = function.interface();
        let number = interface.interface_number();
        let name = interface.string();
        interface.alt_setting(
            INTERFACE_CLASS,
            INTERFACE_SUBCLASS,
            INTERFACE_PROTOCOL,
            Some(name),
        );
        Self {
            number,
            name,
            requests,
        }
    }
}

impl Handler for ResetInterface<'_> {
    /// Accepts a reboot request to the reset interface and rejects every other request to it.
    fn control_out(&mut self, request: Request, _data: &[u8]) -> Option<OutResponse> {
        let to_this_interface = request.request_type == RequestType::Class
            && request.recipient == Recipient::Interface
            && request.index == u16::from(self.number.0);
        if !to_this_interface {
            return None;
        }
        Some(
            match RebootToBootloader::decode(request.request, request.value) {
                Some(reboot) => {
                    self.requests.signal(reboot);
                    OutResponse::Accepted
                }
                None => OutResponse::Rejected,
            },
        )
    }

    /// Returns the name of the reset interface.
    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        (index == self.name).then_some(INTERFACE_NAME)
    }
}

/// Runs the USB device and reboots into the USB bootloader when a host asks for it.
///
/// The descriptors and the handler live in this task, which never returns. The device and the
/// reboot wait in the same task and never block, so the light task keeps its cadence with or
/// without a host.
#[embassy_executor::task]
pub async fn usb_task(mut usb: Peri<'static, USB>) -> ! {
    let mut config_descriptor = [0; CONFIG_DESCRIPTOR_LEN];
    let mut bos_descriptor = [0; BOS_DESCRIPTOR_LEN];
    let mut msos_descriptor = [0; MSOS_DESCRIPTOR_LEN];
    let mut control_buf = [0; CONTROL_BUF_LEN];
    let requests = Requests::new();
    // Declared before the builder, which keeps a reference to the handler.
    let mut reset = None;

    let mut builder = Builder::new(
        Driver::new(usb.reborrow(), Irqs),
        CONFIG,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );
    // Device-level features, as the device has a single function: Windows binds WinUSB to all of
    // it.
    builder.msos_descriptor(windows_version::WIN8_1, MSOS_VENDOR_CODE);
    builder.msos_feature(CompatibleIdFeatureDescriptor::new(WINUSB, ""));
    builder.msos_feature(RegistryPropertyFeatureDescriptor::new(
        INTERFACE_GUIDS_NAME,
        PropertyData::RegMultiSz(&[INTERFACE_GUID]),
    ));
    let handler = reset.insert(ResetInterface::add_to(&mut builder, &requests));
    builder.handler(handler);
    let mut device = builder.build();

    join(device.run(), reboot_on_request(&requests)).await.0
}

/// Reboots into the USB bootloader after every accepted request.
///
/// The firmware keeps running for [`STATUS_STAGE_GRACE`] first, while the host completes the
/// transfer. `reset_to_usb_boot` has the boot ROM reboot the chip through the watchdog and only
/// returns if the ROM did not; the lights then carry on and the next request tries again.
async fn reboot_on_request(requests: &Requests) -> ! {
    loop {
        let reboot = requests.wait().await;
        Timer::after(STATUS_STAGE_GRACE).await;
        rom_data::reset_to_usb_boot(NO_ACTIVITY_LED, reboot.disable_interface_mask());
    }
}
