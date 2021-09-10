# quad-gamepad

Light-weight and opinionated gamepad handling libarary.

- [x] Windows: xinput  
- [x] Linux: evdev  
- [x] Mac: iokit  
- [ ] Web: ?
- [ ] Android: ?  
- [ ] IOS: ?  

## Attribution

quad-gamepad is a fork of https://github.com/unrust/gamepad-rs 
both API and implementation diverged quite a lot, so right now the fork is published on crates as `quad-gamepad`.

linux's evdev implementation is based on https://github.com/glfw/glfw/blob/master/src/linux_joystick.c
