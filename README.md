# Device Honey Pot

This repo is part of this [project](https://github.com/bWanShiTong/reverse-engineering-whoop). Propose of this is too simulate Whoop Device, in order to see what data is sent to device from app, in order to control device.

## Todo:

- [ ] Device needs to be detected by Whoop APP
- [x] (`0x180A`) Device information
    - [ ] (`0x2a29`)  Manufacturer Name String should return `WHOOP Inc.`
- [x] (`0x1800`) Generic Access
    - [x] (`0x2A00`) Device Name should return device name
    - [x] (`0x2A01`) Appearance doesn't return anything but can be read
    - [ ] (`0x2A04`) Peripheral preferred connection parameters should be read, and return `0x06003C0000009001`
    - [x] (`0x2AA6`) Central address resolution should be read and return `0x01`
- [x] (`0x1801`) Generic Attribute 
    - [x] (`0x2A05`) Service Change, indicate and read
- [x] (`0x181E`) Bond Management 
    - [x] (`0x2AA5`) Bond Management Feature, read and returns `0x200802`
    - [x] (`0x2AA4`) Bond Management Control Point, write and read
- [x] (`0x180D`) Heart Rate
    - [x] (`0x2A37`) Heart rate measurement, notifiable and readable
- [x] (`0x180F`) Battery Level
    - [x] (`0x2A19`) Battery Level, read and notifiable
- [x] (`61080001-8d6d-82b8-614a-1c8cb0f8dcc6`) Custom Service
    - [x] (`61080002-8d6d-82b8-614a-1c8cb0f8dcc6`) Writeable and Writeable without response, log here for data sent
    - [x] (`61080003-8d6d-82b8-614a-1c8cb0f8dcc6`) Notifiable
    - [x] (`61080004-8d6d-82b8-614a-1c8cb0f8dcc6`) Notifiable
    - [x] (`61080005-8d6d-82b8-614a-1c8cb0f8dcc6`) Notifiable
    - [x] (`61080007-8d6d-82b8-614a-1c8cb0f8dcc6`) Notifiable