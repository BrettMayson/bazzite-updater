use serde::Deserialize;

pub struct Config {
    machines: Vec<Machine>,
}

impl Config {
    pub fn from_file(file: ConfigFile) -> Self {
        Self {
            machines: file.machines.into_iter().map(Machine::from_file).collect(),
        }
    }

    pub fn read_file(path: &str) -> Self {
        Self::from_file(
            serde_yaml::from_reader(std::fs::File::open(path).expect("Failed to open config file"))
                .expect("Failed to parse config file"),
        )
    }

    pub fn servers(&self) -> &Vec<Machine> {
        &self.machines
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub machines: Vec<MachineFile>,
}

#[derive(Debug, Clone)]
pub struct Machine {
    ssh: String,
    mac: Option<String>,
    cron: String,
    steam: bool,
    /// The number of minuts with no activity before the machine is shutdown
    steam_delay: u64,
    flatpak: bool,
}

impl Machine {
    pub fn from_file(file: MachineFile) -> Self {
        Self {
            ssh: file.ssh,
            mac: file.mac,
            cron: file.cron,
            steam: file.steam.unwrap_or(false),
            steam_delay: {
                let delay = file.steam_delay.unwrap_or(5);
                if delay == 0 {
                    panic!("Steam delay must be greater than 0");
                }
                delay
            },
            flatpak: file.flatpak.unwrap_or(false),
        }
    }

    pub fn ssh(&self) -> &str {
        &self.ssh
    }

    pub fn mac(&self) -> Option<&str> {
        self.mac.as_deref()
    }

    pub fn cron(&self) -> &str {
        &self.cron
    }

    pub fn steam(&self) -> bool {
        self.steam
    }

    pub fn steam_delay(&self) -> u64 {
        self.steam_delay
    }

    pub fn flatpak(&self) -> bool {
        self.flatpak
    }
}

#[derive(Debug, Deserialize)]
pub struct MachineFile {
    pub ssh: String,
    pub mac: Option<String>,
    pub cron: String,
    pub steam: Option<bool>,
    pub steam_delay: Option<u64>,
    pub flatpak: Option<bool>,
}
