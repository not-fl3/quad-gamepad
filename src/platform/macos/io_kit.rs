#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    dead_code
)]

use core_foundation as cf;

pub type IOHIDDeviceRef = *mut libc::c_void;
pub type IOHIDElementRef = *mut libc::c_void;
pub type IOHIDValueRef = *mut libc::c_void;
pub type IOHIDManagerRef = *mut libc::c_void;
pub type IOHIDElementCookie = u32;
pub type IOOptionBits = libc::c_uint;
pub type IOReturn = libc::c_uint;

pub const kIOHIDOptionsTypeNone: IOOptionBits = 0x0;
pub const kHIDPage_GenericDesktop: u32 = 0x1;
pub const kHIDPage_Button: u32 = 0x09;
pub const kHIDPage_Consumer: u32 = 0x0C;

pub const kHIDUsage_GD_Joystick: u32 = 0x04;
pub const kHIDUsage_GD_GamePad: u32 = 0x05;
pub const kHIDUsage_GD_MultiAxisController: u32 = 0x08;

pub const kHIDUsage_GD_X: u32 = 0x30;
pub const kHIDUsage_GD_Y: u32 = 0x31;
pub const kHIDUsage_GD_Z: u32 = 0x32;
pub const kHIDUsage_GD_Rx: u32 = 0x33;
pub const kHIDUsage_GD_Ry: u32 = 0x34;
pub const kHIDUsage_GD_Rz: u32 = 0x35;
pub const kHIDUsage_GD_Slider: u32 = 0x36;
pub const kHIDUsage_GD_Dial: u32 = 0x37;
pub const kHIDUsage_GD_Wheel: u32 = 0x38;

pub const kHIDUsage_GD_Hatswitch: u32 = 0x39;

pub const kHIDUsage_GD_Start: u32 = 0x3D;
pub const kHIDUsage_GD_Select: u32 = 0x3E;
pub const kHIDUsage_GD_SystemMainMenu: u32 = 0x85;

pub const kHIDUsage_GD_DPadUp: u32 = 0x90;
pub const kHIDUsage_GD_DPadDown: u32 = 0x91;
pub const kHIDUsage_GD_DPadRight: u32 = 0x92;
pub const kHIDUsage_GD_DPadLeft: u32 = 0x93;

pub const kIOReturnSuccess: libc::c_uint = 0;

pub type IOHIDManagerOptions = IOOptionBits;
pub const kIOHIDManagerOptionNone: IOHIDManagerOptions = 0x0;

pub fn kIOHIDVendorIDKey() -> &'static str {
    "VendorID"
}

pub fn kIOHIDVersionNumberKey() -> &'static str {
    "VersionNumber"
}

pub fn kIOHIDProductKey() -> &'static str {
    "Product"
}

pub fn kIOHIDProductIDKey() -> &'static str {
    "ProductID"
}

pub fn kIOHIDDeviceUsageKey() -> &'static str {
    "DeviceUsage"
}
pub fn kIOHIDDeviceUsagePageKey() -> &'static str {
    "DeviceUsagePage"
}
pub fn kIOHIDPrimaryUsagePageKey() -> &'static str {
    "PrimaryUsagePage"
}
pub fn kIOHIDPrimaryUsageKey() -> &'static str {
    "PrimaryUsage"
}

type IOHIDValueCallback = unsafe extern "C" fn(
    context: *mut libc::c_void,
    result: IOReturn,
    sender: *mut libc::c_void,
    value: IOHIDValueRef,
);
type IOHIDCallback =
    unsafe extern "C" fn(context: *mut libc::c_void, result: IOReturn, sender: *mut libc::c_void);
type IOHIDDeviceCallback = unsafe extern "C" fn(
    context: *mut libc::c_void,
    result: IOReturn,
    sender: *mut libc::c_void,
    device: IOHIDDeviceRef,
);

extern "C" {
    // pub fn IOHIDDeviceGetValueWithCallback(
    //     device: IOHIDDeviceRef,
    //     element: IOHIDElementRef,
    //     pValue: *mut IOHIDValueRef,
    //     timeout: cf::date::CFTimeInterval,
    //     callback: IOHIDValueCallback,
    //     context: *mut libc::c_void,
    // ) -> IOReturn;
    pub fn IOHIDManagerUnscheduleFromRunLoop(
        manager: IOHIDManagerRef,
        runLoop: cf::runloop::CFRunLoopRef,
        runLoopMode: cf::string::CFStringRef,
    );
    pub fn IOHIDManagerClose(manager: IOHIDManagerRef, options: IOHIDManagerOptions) -> IOReturn;
    pub fn IOHIDDeviceGetValue(
        device: IOHIDDeviceRef,
        element: IOHIDElementRef,
        pValue: *mut IOHIDValueRef,
    ) -> IOReturn;
    pub fn IOHIDValueGetIntegerValue(value: IOHIDValueRef) -> cf::base::CFIndex;
    pub fn IOHIDDeviceGetProperty(
        device: IOHIDDeviceRef,
        key: cf::string::CFStringRef,
    ) -> cf::base::CFTypeRef;
    pub fn IOHIDDeviceRegisterRemovalCallback(
        device: IOHIDDeviceRef,
        callback: IOHIDCallback,
        context: *mut libc::c_void,
    );
    pub fn IOHIDElementGetTypeID() -> cf::base::CFTypeID;
    pub fn IOHIDElementGetCookie(element: IOHIDElementRef) -> IOHIDElementCookie;
    pub fn IOHIDElementGetUsagePage(element: IOHIDElementRef) -> u32;
    pub fn IOHIDElementGetUsage(element: IOHIDElementRef) -> u32;
    pub fn IOHIDElementGetLogicalMin(element: IOHIDElementRef) -> cf::base::CFIndex;
    pub fn IOHIDElementGetLogicalMax(element: IOHIDElementRef) -> cf::base::CFIndex;
    pub fn IOHIDDeviceCopyMatchingElements(
        device: IOHIDDeviceRef,
        matching: cf::dictionary::CFDictionaryRef,
        options: IOOptionBits,
    ) -> cf::array::CFArrayRef;
    pub fn IOHIDManagerCreate(
        allocator: cf::base::CFAllocatorRef,
        options: IOHIDManagerOptions,
    ) -> IOHIDManagerRef;
    pub fn IOHIDManagerOpen(manager: IOHIDManagerRef, options: IOHIDManagerOptions) -> IOReturn;
    pub fn IOHIDManagerSetDeviceMatchingMultiple(
        manager: IOHIDManagerRef,
        multiple: cf::array::CFArrayRef,
    );
    pub fn IOHIDManagerRegisterDeviceMatchingCallback(
        manager: IOHIDManagerRef,
        callback: IOHIDDeviceCallback,
        context: *mut libc::c_void,
    );
    pub fn IOHIDManagerScheduleWithRunLoop(
        manager: IOHIDManagerRef,
        runLoop: cf::runloop::CFRunLoopRef,
        runLoopMode: cf::string::CFStringRef,
    );
}
