use super::types::{GroupState, KeyState, SnapKeyPreset};

#[derive(Clone, Debug)]
pub struct SnapKeyState {
    pub preset: SnapKeyPreset,
    pub keys: [KeyState; 256],
    pub groups: [GroupState; 4],
}

impl Default for SnapKeyState {
    fn default() -> Self {
        Self {
            preset: SnapKeyPreset::Off,
            keys: [KeyState::default(); 256],
            groups: [GroupState::default(); 4],
        }
    }
}

impl SnapKeyState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_preset<F: FnMut(u16, bool)>(&mut self, preset: SnapKeyPreset, mut send_key: F) {
        // First, release any active simulated keys from previous preset
        for group in &mut self.groups {
            if group.active_key != 0 {
                send_key(group.active_key, false);
            }
            group.previous_key = 0;
            group.active_key = 0;
        }

        self.preset = preset;
        self.keys = [KeyState::default(); 256];
        self.groups = [GroupState::default(); 4];

        match preset {
            SnapKeyPreset::Off => {}
            SnapKeyPreset::Wasd => {
                // Group 0: Strafe (A = 65, D = 68)
                self.register_pair(0, 65, 68);
                // Group 1: Movement (S = 83, W = 87)
                self.register_pair(1, 83, 87);
            }
            SnapKeyPreset::ArrowKeys => {
                // Group 0: Horizontal (Left = 37, Right = 39)
                self.register_pair(0, 37, 39);
                // Group 1: Vertical (Up = 38, Down = 40)
                self.register_pair(1, 38, 40);
            }
            SnapKeyPreset::Esdf => {
                // Group 0: Strafe (S = 83, F = 70)
                self.register_pair(0, 83, 70);
                // Group 1: Movement (D = 68, E = 69)
                self.register_pair(1, 68, 69);
            }
            SnapKeyPreset::Azerty => {
                // Group 0: Strafe (Q = 81, D = 68)
                self.register_pair(0, 81, 68);
                // Group 1: Movement (S = 83, Z = 90)
                self.register_pair(1, 83, 90);
            }
        }
    }

    fn register_pair(&mut self, group: u8, key1: u16, key2: u16) {
        if let Some(k1) = self.keys.get_mut(key1 as usize) {
            k1.registered = true;
            k1.group = group;
            k1.key_down = false;
        }
        if let Some(k2) = self.keys.get_mut(key2 as usize) {
            k2.registered = true;
            k2.group = group;
            k2.key_down = false;
        }
    }

    #[must_use]
    pub fn is_registered(&self, vk: u16) -> bool {
        self.keys.get(vk as usize).is_some_and(|k| k.registered)
    }

    pub fn handle_key_down<F: FnMut(u16, bool)>(&mut self, vk: u16, mut send_key: F) {
        let Some(key_info) = self.keys.get_mut(vk as usize) else {
            return;
        };
        if !key_info.registered {
            return;
        }

        let group_idx = key_info.group as usize;
        if !key_info.key_down {
            key_info.key_down = true;
            send_key(vk, true);

            if let Some(group) = self.groups.get_mut(group_idx) {
                if group.active_key == 0 || group.active_key == vk {
                    group.active_key = vk;
                } else {
                    group.previous_key = group.active_key;
                    group.active_key = vk;
                    send_key(group.previous_key, false);
                }
            }
        } else if let Some(group) = self.groups.get(group_idx) {
            // Typematic auto-repeat: when a key is already marked down and hardware generates
            // repeated keydown messages, forward the event if this key is the active direction.
            if group.active_key == vk {
                send_key(vk, true);
            }
        }
    }

    pub fn handle_key_up<F: FnMut(u16, bool)>(&mut self, vk: u16, mut send_key: F) {
        let Some(key_info) = self.keys.get_mut(vk as usize) else {
            return;
        };
        if !key_info.registered {
            return;
        }

        let group_idx = key_info.group as usize;
        let Some(group) = self.groups.get_mut(group_idx) else {
            return;
        };

        if group.previous_key == vk && !key_info.key_down {
            group.previous_key = 0;
        }

        if key_info.key_down {
            key_info.key_down = false;
            if group.active_key == vk && group.previous_key != 0 {
                send_key(vk, false);
                group.active_key = group.previous_key;
                let to_press = group.active_key;
                group.previous_key = 0;
                send_key(to_press, true);
            } else {
                group.previous_key = 0;
                if group.active_key == vk {
                    group.active_key = 0;
                }
                send_key(vk, false);
            }
        }
    }

    pub fn sync_ghost_keys<Q, F>(&mut self, exclude_vk: u16, mut is_key_down: Q, mut send_key: F)
    where
        Q: FnMut(u16) -> bool,
        F: FnMut(u16, bool),
    {
        for group in &mut self.groups {
            let prev = group.previous_key;
            if prev != 0 && prev != exclude_vk && !is_key_down(prev) {
                group.previous_key = 0;
                if let Some(info) = self.keys.get_mut(prev as usize) {
                    info.key_down = false;
                }
            }
        }

        for i in 0..self.groups.len() {
            let active = self.groups[i].active_key;
            if active != 0 && active != exclude_vk && !is_key_down(active) {
                self.handle_key_up(active, &mut send_key);
            }
        }
    }
}
