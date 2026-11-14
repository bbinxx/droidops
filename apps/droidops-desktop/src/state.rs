use droidops_core::Device;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Dashboard,
    Applications,
    Files,
    Gallery,
    Shell,
    Recovery,
    Fastboot,
    Settings,
}

impl Screen {
    pub const ALL: [Self; 8] = [
        Self::Dashboard,
        Self::Applications,
        Self::Files,
        Self::Gallery,
        Self::Shell,
        Self::Recovery,
        Self::Fastboot,
        Self::Settings,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Applications => "Applications",
            Self::Files => "Files",
            Self::Gallery => "Gallery",
            Self::Shell => "ADB shell",
            Self::Recovery => "Recovery sideload",
            Self::Fastboot => "Power & Fastboot",
            Self::Settings => "Settings",
        }
    }
    pub fn subtitle(self) -> &'static str {
        match self {
            Self::Dashboard => "A clear view of your Android device.",
            Self::Applications => "Install and manage Android packages.",
            Self::Files => "Browse shared storage and transfer files.",
            Self::Gallery => "Photos from DCIM and Pictures.",
            Self::Shell => "Run a command on the selected device.",
            Self::Recovery => "Apply a ZIP update from Android recovery.",
            Self::Fastboot => "Restart your device into the mode you need.",
            Self::Settings => "Make this workspace your own.",
        }
    }
}

/// A generation changes on every selection boundary, including A → B → A.
#[derive(Default)]
pub struct Session {
    pub device: Option<Device>,
    pub generation: u64,
}

impl Session {
    pub fn select(&mut self, device: Option<Device>) {
        self.generation = self.generation.wrapping_add(1);
        self.device = device;
    }

    pub fn accepts(&self, generation: u64) -> bool {
        self.generation == generation
    }

    /// Losing a device or a state transition invalidates selection and confirmation.
    pub fn reconcile(&mut self, devices: &[Device]) -> bool {
        if let Some(selected) = &self.device {
            if !devices
                .iter()
                .any(|d| d.key() == selected.key() && d.state == selected.state)
            {
                self.select(None);
                return true;
            }
        }
        false
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub dark: bool,
    pub scale: f32,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            dark: false,
            scale: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidops_core::Transport;
    fn device(serial: &str) -> Device {
        Device {
            serial: serial.into(),
            model: "Fixture".into(),
            state: "device".into(),
            transport: Transport::Adb,
        }
    }

    #[test]
    fn discovery_never_auto_selects_and_reconnect_does_not_restore_consent() {
        let mut session = Session::default();
        assert!(!session.reconcile(&[device("a")]));
        assert!(session.device.is_none());
        session.select(Some(device("a")));
        let generation = session.generation;
        assert!(session.reconcile(&[]));
        assert!(!session.accepts(generation));
        session.reconcile(&[device("a")]);
        assert!(session.device.is_none());
    }

    #[test]
    fn switching_back_to_same_device_still_rejects_stale_results() {
        let mut session = Session::default();
        session.select(Some(device("a")));
        let old = session.generation;
        session.select(Some(device("b")));
        session.select(Some(device("a")));
        assert!(!session.accepts(old));
        assert!(session.accepts(session.generation));
    }

    #[test]
    fn unauthorized_transition_invalidates_selection() {
        let mut session = Session::default();
        session.select(Some(device("a")));
        let mut unauthorized = device("a");
        unauthorized.state = "unauthorized".into();
        assert!(session.reconcile(&[unauthorized]));
        assert!(session.device.is_none());
    }
}
