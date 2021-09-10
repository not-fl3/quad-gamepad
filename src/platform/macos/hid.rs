use core_foundation as cf;
use core_foundation::array as cf_array;
use core_foundation::array::{CFArray, CFArrayRef};

use core_foundation::base::{CFGetTypeID, CFRetain, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::{kCFNumberSInt32Type, CFNumber, CFNumberGetValue};
use core_foundation::runloop::{
    kCFRunLoopRunHandledSource, CFRunLoopGetCurrent, CFRunLoopRunInMode,
};
use core_foundation::string::CFString;

use libc::c_void;
use std::cell::RefCell;
use std::ptr;
use std::rc::{Rc, Weak};

use super::io_kit::{self, *};

fn gamepad_rs_runloop_mode() -> CFString {
    "GamepadRS".into()
}

#[derive(Debug)]
pub enum Error {
    Unknown(String),
}

type HIDResult<T> = Result<T, Error>;

pub struct HIDStateContext {
    state: Rc<HIDState>,
}

pub struct HIDState {
    hidman: IOHIDManagerRef,
    pub devices: RefCell<Vec<Weak<RefCell<Device>>>>,
}

pub struct HID {
    state: Weak<HIDState>,
}

impl Drop for HID {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            if state.hidman != ptr::null_mut() {
                {
                    let devices = state.devices.borrow();
                    for d in devices.iter() {
                        if let Some(dd) = d.upgrade() {
                            dd.borrow_mut().remove_from_runloop();
                        }
                    }
                }
                state.devices.borrow_mut().clear();

                unsafe {
                    let current_loop = CFRunLoopGetCurrent();
                    let mode = gamepad_rs_runloop_mode();
                    if current_loop != ptr::null_mut() {
                        IOHIDManagerUnscheduleFromRunLoop(
                            state.hidman,
                            current_loop as _,
                            mode.as_CFType().as_CFTypeRef() as _,
                        );
                    }
                    IOHIDManagerClose(state.hidman, kIOHIDOptionsTypeNone);
                }
            }
        }
    }
}

fn create_hid_device_mach_dictionary(page: u32, usage: u32) -> CFDictionary<CFString, CFNumber> {
    let page = CFNumber::from(page as i32);
    let usage = CFNumber::from(usage as i32);

    let page_key = CFString::from(kIOHIDDeviceUsagePageKey());
    let usage_key = CFString::from(kIOHIDDeviceUsageKey());

    CFDictionary::from_CFType_pairs(&[(page_key, page), (usage_key, usage)])
}

#[derive(Debug)]
pub struct HIDElement {
    usage: u32,
    page: u32,
    ref_elem: IOHIDElementRef,
    cookie: IOHIDElementCookie,

    min_report: i32,
    max_report: i32,
}

impl HIDElement {
    fn query_axis(&mut self, device_ref: IOHIDDeviceRef, min: i32, max: i32) -> Option<i32> {
        let device_scale = (max - min) as f32;
        let read_scale = (self.max_report - self.min_report) as f32;

        let state = self.query_state(device_ref);
        if state.is_none() {
            return None;
        }
        let state = state.unwrap();

        Some((((state - self.min_report) as f32) * device_scale / read_scale) as i32 + min)
    }

    fn query_state(&mut self, device_ref: IOHIDDeviceRef) -> Option<i32> {
        use std::mem;

        if device_ref == ptr::null_mut() || self.ref_elem == ptr::null_mut() {
            return None;
        }
        let mut value_ref: IOHIDValueRef = ptr::null_mut();

        unsafe {
            if IOHIDDeviceGetValue(device_ref, self.ref_elem, mem::transmute(&mut value_ref))
                == kIOReturnSuccess
            {
                let value = IOHIDValueGetIntegerValue(value_ref) as i32;

                // record min and max for auto calibration
                self.min_report = self.min_report.min(value);
                self.max_report = self.max_report.max(value);

                return Some(value);
            }
        }

        None
    }
}

struct DeviceContext {
    device: Rc<RefCell<Device>>,
}

#[derive(Default, Debug)]
pub struct DeviceState {
    pub sequence: usize,
    pub digital_state: Vec<bool>,
    pub analog_state: Vec<f32>,
}

#[derive(Debug)]
pub struct Device {
    usage: i32,
    page: i32,
    device: IOHIDDeviceRef,

    pub name: String,
    pub guid: String,
    pub axes: Vec<HIDElement>,
    pub hats: Vec<HIDElement>,
    pub buttons: Vec<HIDElement>,

    pub state: DeviceState,
}

impl Drop for Device {
    fn drop(&mut self) {
        self.remove_from_runloop();
    }
}

unsafe fn get_property_i32(dev: IOHIDDeviceRef, s: &'static str) -> Option<i32> {
    use std::mem;

    let key = CFString::from(s);
    let ref_cf = IOHIDDeviceGetProperty(dev, key.as_CFTypeRef() as _);
    if ref_cf == ptr::null() {
        return None;
    }

    let mut value: i32 = 0;
    let ok = CFNumberGetValue(ref_cf as _, kCFNumberSInt32Type, mem::transmute(&mut value));

    if ok {
        Some(value)
    } else {
        None
    }
}

unsafe fn get_property_str(dev: IOHIDDeviceRef, s: &'static str) -> Option<String> {
    let key = CFString::from(s);
    let ref_cf = IOHIDDeviceGetProperty(dev, key.as_CFTypeRef() as _);
    if ref_cf == ptr::null() {
        return None;
    }

    let cf_str = CFString::wrap_under_create_rule(ref_cf as _);
    Some(cf_str.to_string())
}

impl Device {
    fn remove_from_runloop(&mut self) {
        if self.device != ptr::null_mut() {
            use std::mem;
            unsafe {
                IOHIDDeviceRegisterRemovalCallback(
                    self.device,
                    mem::transmute::<*const (), _>(ptr::null()),
                    ptr::null_mut(),
                );

                // We only work in main thread, so that it
                // seem to no need to schedule it
                //
                // TODO: Find out how to know do it properly.

                // let current_loop = CFRunLoopGetCurrent();
                // let mode = gamepad_rs_runloop_mode();
                // // io_kit::IOHIDDeviceUnscheduleFromRunLoop(
                //     self.device,
                //     current_loop as _,
                //     mode.as_CFType().as_CFTypeRef() as _,
                // );
            }

            self.device = ptr::null_mut();
        }
    }

    fn contain_element(&self, cookie: IOHIDElementCookie) -> bool {
        self.axes.iter().any(|elm| elm.cookie == cookie)
            || self.buttons.iter().any(|elm| elm.cookie == cookie)
            || self.hats.iter().any(|elm| elm.cookie == cookie)
    }

    unsafe fn add_element(&mut self, ref_elem: IOHIDElementRef) {
        if ref_elem == ptr::null_mut() {
            return;
        }

        let elem_type_id = CFGetTypeID(ref_elem as _);
        if elem_type_id != IOHIDElementGetTypeID() {
            return;
        }

        let cookie = IOHIDElementGetCookie(ref_elem);

        if self.contain_element(cookie) {
            return;
        }

        let page = IOHIDElementGetUsagePage(ref_elem);
        let usage = IOHIDElementGetUsage(ref_elem);

        let mut target = None;

        if page == kHIDPage_GenericDesktop {
            match usage {
                io_kit::kHIDUsage_GD_X
                | io_kit::kHIDUsage_GD_Y
                | io_kit::kHIDUsage_GD_Z
                | io_kit::kHIDUsage_GD_Rx
                | io_kit::kHIDUsage_GD_Ry
                | io_kit::kHIDUsage_GD_Rz
                | io_kit::kHIDUsage_GD_Slider
                | io_kit::kHIDUsage_GD_Dial
                | io_kit::kHIDUsage_GD_Wheel => {
                    target = Some(&mut self.axes);
                }
                io_kit::kHIDUsage_GD_Hatswitch => {
                    target = Some(&mut self.hats);
                }
                io_kit::kHIDUsage_GD_DPadUp
                | io_kit::kHIDUsage_GD_DPadDown
                | io_kit::kHIDUsage_GD_DPadRight
                | io_kit::kHIDUsage_GD_DPadLeft
                | io_kit::kHIDUsage_GD_Start
                | io_kit::kHIDUsage_GD_Select
                | io_kit::kHIDUsage_GD_SystemMainMenu => {
                    target = Some(&mut self.buttons);
                }
                _ => {}
            }
        } else if page == io_kit::kHIDPage_Button || page == io_kit::kHIDPage_Consumer {
            target = Some(&mut self.buttons);
        }

        if let Some(target) = target {
            target.push(HIDElement {
                usage,
                page,
                ref_elem,
                cookie,
                min_report: IOHIDElementGetLogicalMin(ref_elem) as i32,
                max_report: IOHIDElementGetLogicalMax(ref_elem) as i32,
            });
        }
    }

    fn add_elements(&mut self, array: CFArrayRef) {
        use self::cf_array::*;

        let count = unsafe { CFArrayGetCount(array) };

        for i in 0..count {
            unsafe {
                let ref_elem: IOHIDElementRef = CFArrayGetValueAtIndex(array, i) as _;
                self.add_element(ref_elem);
            }
        }
    }

    fn from_raw_dev(dev: IOHIDDeviceRef) -> Option<Rc<RefCell<Device>>> {
        unsafe {
            let page = {
                if let Some(val) = get_property_i32(dev, kIOHIDPrimaryUsagePageKey()) {
                    val
                } else {
                    return None;
                }
            };
            //  Filter device list to non-keyboard/mouse stuff
            if page != kHIDPage_GenericDesktop as i32 {
                return None;
            }

            let usage = {
                if let Some(val) = get_property_i32(dev, kIOHIDPrimaryUsageKey()) {
                    val
                } else {
                    return None;
                }
            };

            //  Filter device list to non-keyboard/mouse stuff
            if usage != kHIDUsage_GD_Joystick as i32
                && usage != kHIDUsage_GD_GamePad as i32
                && usage != kHIDUsage_GD_MultiAxisController as i32
            {
                return None;
            }

            let name = get_property_str(dev, kIOHIDProductKey()).unwrap_or("unknown".to_owned());

            let vendor = get_property_i32(dev, kIOHIDVendorIDKey()).unwrap_or(0);
            let product_id = get_property_i32(dev, kIOHIDProductIDKey()).unwrap_or(0);
            let version = get_property_i32(dev, kIOHIDVersionNumberKey()).unwrap_or(0);

            #[rustfmt::skip]
            let guid = if vendor != 0 && product_id != 0 {
                format!("03000000{:02x}{:02x}0000{:02x}{:02x}0000{:02x}{:02x}0000",
                        vendor as u8, (vendor >> 8) as u8,
                        product_id as u8,  (product_id >> 8) as u8,
                        version as u8, (version >> 8) as u8)
            } else {
                let name = name.as_bytes();
                format!("05000000{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}00",
                        name[0], name[1], name[2], name[3],
                        name[4], name[5], name[6], name[7],
                        name[8], name[9], name[10])
            };

            let mut device = Device {
                usage,
                page,
                guid,
                name,
                device: dev,
                hats: Vec::new(),
                axes: Vec::new(),
                buttons: Vec::new(),
                state: Default::default(),
            };

            let array_cf = IOHIDDeviceCopyMatchingElements(dev, ptr::null(), kIOHIDOptionsTypeNone);

            if array_cf != ptr::null() {
                device.add_elements(array_cf as _);

                // sort all elements by usage
                device.buttons.sort_by(|a, b| a.usage.cmp(&b.usage));
                device.axes.sort_by(|a, b| a.usage.cmp(&b.usage));
                device.hats.sort_by(|a, b| a.usage.cmp(&b.usage));
            }

            // TODO:
            // Handle Mapping?
            // https://github.com/spurious/SDL-mirror/blob/93215c489ac11a2b24b2e2665ee729431fdf537c/src/joystick/darwin/SDL_sysjoystick.c#L458

            let device = Rc::new(RefCell::new(device));
            let device_ctx = Box::new(DeviceContext {
                device: device.clone(),
            });

            // Get notified when this device is disconnected.
            IOHIDDeviceRegisterRemovalCallback(
                device.borrow().device,
                joystick_device_was_removed_cb,
                Box::into_raw(device_ctx) as _,
            );

            // We only work in main thread, so that it
            // seem to no need to schedule it
            //
            // TODO: Find out how to know do it properly.

            // let run_loop = CFRunLoopGetCurrent();
            // let mode = gamepad_rs_runloop_mode();
            // io_kit::IOHIDDeviceScheduleWithRunLoop(
            //     dev,
            //     run_loop as _,
            //     mode.as_CFType().as_CFTypeRef() as _,
            // );

            Some(device)
        }
    }
}

extern "C" fn joystick_device_was_removed_cb(
    context: *mut c_void,
    _res: IOReturn,
    _sender: *mut c_void,
) {
    let dev: *mut DeviceContext = context as *mut DeviceContext;

    unsafe {
        let b = Box::from_raw(dev);
        b.device.borrow_mut().device = ptr::null_mut();
    }
}

extern "C" fn joystick_device_was_added_cb(
    context: *mut c_void,
    res: IOReturn,
    _sender: *mut c_void,
    device: IOHIDDeviceRef,
) {
    if res != kIOReturnSuccess {
        return;
    }

    // cast the context back to HID
    let hid_state_ctx: &mut HIDStateContext = unsafe { &mut *(context as *mut HIDStateContext) };

    if hid_state_ctx.state.already_known(device) {
        // IOKit sent us a duplicate
        return;
    }

    if let Some(dev) = Device::from_raw_dev(device) {
        hid_state_ctx
            .state
            .devices
            .borrow_mut()
            .push(Rc::downgrade(&dev));
    }
}

impl HIDState {
    pub fn already_known(&self, device: IOHIDDeviceRef) -> bool {
        self.devices.borrow().iter().any(|dev| {
            if let Some(p) = dev.upgrade() {
                p.borrow().device == device
            } else {
                false
            }
        })
    }
}

impl HID {
    // Return number of devices
    pub fn num_devices(&self) -> usize {
        let mut n = 0;

        let state = self.hid_state();
        let devices = state.devices.borrow();

        for dev in devices.iter() {
            if let Some(_) = dev.upgrade() {
                n += 1;
            }
        }

        n
    }

    // Query the new devices is inserted or not
    pub fn detect_devices(&mut self) {
        // Remove all empty weak pointer
        {
            let state = self.hid_state();
            let mut devices = state.devices.borrow_mut();
            devices.retain(|p| p.upgrade().is_some());
        }

        unsafe {
            let mode = gamepad_rs_runloop_mode();
            while CFRunLoopRunInMode(mode.as_CFTypeRef() as _, 0.0, 1) == kCFRunLoopRunHandledSource
            {
                /* no-op. Pending callbacks will fire in CFRunLoopRunInMode(). */
            }
        }
    }

    pub fn update(&mut self, dev_index: usize) {
        let state = self.hid_state();
        let devices = state.devices.borrow();
        if dev_index >= devices.len() {
            return;
        }

        let dev = &devices[dev_index].upgrade();
        if dev.is_none() {
            return;
        }

        let dev = dev.as_ref().unwrap();
        let mut dev_bor = dev.borrow_mut();
        let device_ref = dev_bor.device;

        let mut new_state = DeviceState::default();

        for btn in dev_bor.buttons.iter_mut() {
            if let Some(state) = btn.query_state(device_ref) {
                new_state.digital_state.push(state != 0);
            }
        }

        for axis in dev_bor.axes.iter_mut() {
            if let Some(state) = axis.query_axis(device_ref, -32768, 32767) {
                new_state.analog_state.push(state as f32 / 32768.0);
            }
        }

        dev_bor.state = new_state;
    }

    pub fn new() -> HIDResult<HID> {
        let hidman = unsafe {
            let hidman =
                IOHIDManagerCreate(cf::base::kCFAllocatorDefault as _, kIOHIDManagerOptionNone);

            if kIOReturnSuccess != IOHIDManagerOpen(hidman, kIOHIDOptionsTypeNone) {
                return Err(Error::Unknown("Fail to open HID Manager".to_owned()));
            }

            CFRetain(hidman as _);

            hidman
        };

        let mut hid = HID { state: Weak::new() };

        hid.config_manager(Rc::new(HIDState {
            hidman,
            devices: RefCell::new(Vec::new()),
        }));

        Ok(hid)
    }

    pub fn hid_state(&self) -> Rc<HIDState> {
        self.state.upgrade().unwrap()
    }

    fn config_manager(&mut self, state: Rc<HIDState>) {
        self.state = Rc::downgrade(&state);

        unsafe {
            let array = CFArray::from_CFTypes(&[
                create_hid_device_mach_dictionary(kHIDPage_GenericDesktop, kHIDUsage_GD_Joystick),
                create_hid_device_mach_dictionary(kHIDPage_GenericDesktop, kHIDUsage_GD_GamePad),
                create_hid_device_mach_dictionary(
                    kHIDPage_GenericDesktop,
                    kHIDUsage_GD_MultiAxisController,
                ),
            ]);

            let runloop = CFRunLoopGetCurrent();

            let state_ctx = Box::new(HIDStateContext {
                state: state.clone(),
            });

            IOHIDManagerSetDeviceMatchingMultiple(state.hidman, array.as_CFTypeRef() as _);
            IOHIDManagerRegisterDeviceMatchingCallback(
                state.hidman,
                joystick_device_was_added_cb,
                Box::into_raw(state_ctx) as _,
            );

            let mode = gamepad_rs_runloop_mode();
            IOHIDManagerScheduleWithRunLoop(state.hidman, runloop as _, mode.as_CFTypeRef() as _);

            // joystick_device_was_added_cb will be called if there are any devices
            while CFRunLoopRunInMode(mode.as_CFTypeRef() as _, 0.0, 1) == kCFRunLoopRunHandledSource
            {
                /* no-op. Callback fires once per existing device. */
            }
        }
    }
}
