use super::JoystickType;

pub trait Tag {
    fn id(&self) -> u8;

    fn data(&self) -> Vec<u8>;

    fn construct(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.id());
        buf.extend(self.data());
        buf.insert(0, buf.len() as u8);

        buf
    }
}

pub struct JoystickDescriptor {
    index: u8,
    is_xbox: bool,
    joystick_type: JoystickType,
    name: String,
    //TODO: finish later im tired :screm:
}