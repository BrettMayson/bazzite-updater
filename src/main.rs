mod config;

use config::Machine;
use openssh::Session;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, error, info, warn};

struct UpdateSession {
    session: Session,
    machine: Machine,
    did_wake: bool,
}

impl UpdateSession {
    async fn new(machine: Machine) -> Result<Self, String> {
        fn wol(mac: &str) {
            info!("Sending WOL packet to {:?}", mac);
            let wol = wakey::WolPacket::from_string(mac, ':').expect("Failed to parse MAC address");
            if wol.send_magic().is_err() {
                error!("Failed to send the magic packet.");
            }
        }
        info!("Connecting to {}", machine.ssh());
        let mut did_wake = false;
        let mut tries = 0;
        let session = loop {
            match Session::connect(machine.ssh(), openssh::KnownHosts::Accept).await {
                Ok(session) => break session,
                Err(e) => {
                    tries += 1;
                    warn!("Failed to connect to Bazzite: {:?}", e);
                    if tries == 2 {
                        if let Some(mac) = machine.mac() {
                            wol(mac);
                            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                            did_wake = true;
                        } else {
                            info!("No MAC address provided, skipping WOL");
                            return Err(
                                "Could not connect to machine, no MAC address provided".to_string()
                            );
                        }
                    }
                    if tries > 5 {
                        return Err("Could not connect to machine".to_string());
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
            }
        };
        info!("Connected to Bazzite");
        Ok(Self {
            session,
            machine: machine.clone(),
            did_wake,
        })
    }

    async fn command(&self, command: &str) -> std::process::Output {
        self.session
            .command("bash")
            .args(["-c", command])
            .output()
            .await
            .expect("Failed to run command")
    }

    async fn steam(&self) {
        if !self.did_wake {
            info!("Skipping Steam check because the server was not woken up");
            return;
        }
        let mut last_usage = std::time::SystemTime::now();
        loop {
            let usage: u64 = String::from_utf8(self
                .command("netstat -tunap | grep steam | awk '{sum1+=$2; sum2+=$3} END {print sum1 + sum2}'")
                .await
                .stdout).expect("Failed to get usage")
                .trim()
                .parse()
                .expect("Failed to parse usage");
            debug!("Steam usage: {:?}", usage);
            if usage > 250 {
                last_usage = std::time::SystemTime::now();
            } else if last_usage.elapsed().unwrap().as_secs() > self.machine.steam_delay() * 60 {
                error!("No usage, shutting down");
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    async fn flatpak(&self) {
        let output = self.command("sudo flatpak update -y").await;
        if !output.status.success() {
            error!(
                "Failed to update flatpaks: {:?}",
                String::from_utf8(output.stderr).expect("Failed to parse stderr")
            );
        } else {
            info!("Updated flatpaks");
            debug!(
                "{:?}",
                String::from_utf8(output.stdout).expect("Failed to parse stdout")
            );
        }
    }

    pub async fn run(self) {
        if self.machine.flatpak() {
            self.flatpak().await;
        }
        if self.machine.steam() {
            self.steam().await;
        }
        self.close().await;
    }

    async fn close(self) {
        if self.did_wake {
            let result = self.command("sudo shutdown now").await;
            if !result.status.success() {
                error!(
                    "Failed to shutdown: {:?}",
                    String::from_utf8(result.stderr).expect("Failed to parse stderr")
                );
            } else {
                info!("Shutting down");
            }
        }
        if let Err(e) = self.session.close().await {
            error!("Failed to close session: {:?}", e);
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::Config::read_file("config.yaml");

    if std::env::args().any(|arg| arg == "--now") {
        now(config).await;
    } else {
        scheduled(config).await;
    }
}

async fn now(config: config::Config) {
    for machine in config.servers().clone() {
        info!("Running job for `{:?}` now", machine.ssh());
        match UpdateSession::new(machine.clone()).await {
            Ok(session) => session.run().await,
            Err(e) => error!("Failed to create session: {:?}", e),
        }
    }
}

async fn scheduled(config: config::Config) {
    let sched = JobScheduler::new()
        .await
        .expect("Failed to create scheduler");

    for machine in config.servers().clone() {
        info!(
            "Scheduled job for `{:?}` with cron `{}`",
            machine.ssh(),
            machine.cron()
        );
        sched
            .add(
                Job::new_async(machine.clone().cron(), move |_uuid, _l| {
                    let machine = machine.clone();
                    Box::pin(async move {
                        match UpdateSession::new(machine.clone()).await {
                            Ok(session) => session.run().await,
                            Err(e) => error!("Failed to create session: {:?}", e),
                        }
                    })
                })
                .expect("Failed to create job"),
            )
            .await
            .expect("Failed to add job");
    }

    sched.start().await.expect("Failed to start scheduler");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
