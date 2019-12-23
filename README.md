![Version badge](https://img.shields.io/crates/v/ds)

# ds: A library for controlling _FIRST_ Robotics Competition robots

`ds` provides the means to create an FRC driver station, allowing you to enable, and control robots without the use of the official, windows-only driver station. 

This library is very WIP, use at your own risk!

For a project written using this library, see [consoleds](https://gitlab.com/Redrield/consoleds)

# Checklist of things that needeth be done
* [ ] Return a Result from DriverStation::new, and handle the case where the roboRIO can't be found
* [ ] Improve API for sending extra tags over TCP, UDP
