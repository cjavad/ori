use ori_core::event::Key;
use xkbcommon::xkb;

#[allow(unused)]
pub struct XkbKeyboard {
    device_id: i32,
    pub keymap: xkb::Keymap,
    pub state: xkb::State,
}

impl XkbKeyboard {
    #[cfg(x11_platform)]
    pub fn x11_new_from_device(
        connection: &x11rb::xcb_ffi::XCBConnection,
        context: &xkb::Context,
        device_id: i32,
    ) -> Self {
        let keymap = xkb::x11::keymap_new_from_device(
            context,
            connection,
            device_id,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );

        let state = xkb::x11::state_new_from_device(&keymap, connection, device_id);

        Self {
            device_id,
            keymap,
            state,
        }
    }

    #[cfg(x11_platform)]
    pub fn x11_new_core(
        connection: &x11rb::xcb_ffi::XCBConnection,
        context: &xkb::Context,
    ) -> Self {
        let device_id = xkb::x11::get_core_keyboard_device_id(connection);
        Self::x11_new_from_device(connection, context, device_id)
    }

    pub fn device_id(&self) -> i32 {
        self.device_id
    }

    pub fn key_get_utf8(&self, key: xkb::Keycode) -> String {
        self.state.key_get_utf8(key)
    }

    pub fn get_key(&self, key: xkb::Keycode) -> Key {
        let utf8 = self.key_get_utf8(key);

        if !utf8.is_empty() {
            let mut chars = utf8.chars();
            let c = chars.next().unwrap();
            debug_assert!(chars.next().is_none());

            if !c.is_control() {
                return Key::Character(c);
            }
        }

        let keysym = self.state.key_get_one_sym(key);
        crate::platform::linux::xkb::keysym_to_key(keysym)
    }
}
