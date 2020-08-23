![Version badge](https://img.shields.io/crates/v/ds)

# ds: A library for controlling _FIRST_ Robotics Competition robots

`ds` provides the means to create an FRC driver station, allowing you to enable, and control robots without the use of the official, windows-only driver station. 

For a project written using this library, see [Conductor](https://github.com/Redrield/Conductor), a cross-platform driver station written with this library.

The `libDS` subdirectory is a crate exposing a C API around `ds-rs`. 



## Note about the FMS

This crate is intended to create driver stations usable for quick iterations in a shop setting, for users who participate in FRC with a non-Windows computer. Due to this, FMS support is purposely omitted from the protocol stack. The NI Driver Station is the only DS that should be used in a competition setting, and any PRs attempting to add support for the Field Management System to this library will be closed immediately.

