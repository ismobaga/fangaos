/// USB descriptor parsing and structures

/// USB Device Descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub usb_version: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version: u16,
    pub manufacturer_string: u8,
    pub product_string: u8,
    pub serial_number_string: u8,
    pub num_configurations: u8,
}

/// USB Configuration Descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ConfigurationDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_string: u8,
    pub attributes: u8,
    pub max_power: u8,
}

/// USB Interface Descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct InterfaceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface_string: u8,
}

/// USB Endpoint Descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EndpointDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

/// USB HID Descriptor
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HidDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub hid_version: u16,
    pub country_code: u8,
    pub num_descriptors: u8,
    pub report_descriptor_type: u8,
    pub report_descriptor_length: u16,
}

/// Descriptor types
pub mod descriptor_type {
    pub const DEVICE: u8 = 0x01;
    pub const CONFIGURATION: u8 = 0x02;
    pub const STRING: u8 = 0x03;
    pub const INTERFACE: u8 = 0x04;
    pub const ENDPOINT: u8 = 0x05;
    pub const DEVICE_QUALIFIER: u8 = 0x06;
    pub const OTHER_SPEED_CONFIG: u8 = 0x07;
    pub const INTERFACE_POWER: u8 = 0x08;
    pub const HID: u8 = 0x21;
    pub const REPORT: u8 = 0x22;
    pub const PHYSICAL: u8 = 0x23;
}

/// USB class codes
pub mod class_code {
    pub const INTERFACE: u8 = 0x00;
    pub const AUDIO: u8 = 0x01;
    pub const COMM: u8 = 0x02;
    pub const HID: u8 = 0x03;
    pub const PHYSICAL: u8 = 0x05;
    pub const IMAGE: u8 = 0x06;
    pub const PRINTER: u8 = 0x07;
    pub const MASS_STORAGE: u8 = 0x08;
    pub const HUB: u8 = 0x09;
    pub const CDC_DATA: u8 = 0x0A;
    pub const SMART_CARD: u8 = 0x0B;
    pub const CONTENT_SECURITY: u8 = 0x0D;
    pub const VIDEO: u8 = 0x0E;
    pub const PERSONAL_HEALTHCARE: u8 = 0x0F;
    pub const DIAGNOSTIC: u8 = 0xDC;
    pub const WIRELESS: u8 = 0xE0;
    pub const MISCELLANEOUS: u8 = 0xEF;
    pub const VENDOR_SPECIFIC: u8 = 0xFF;
}
