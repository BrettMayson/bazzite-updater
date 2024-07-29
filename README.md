# Bazzite Updater

A docker image to update your bazzite system on a schedule.

## Usage

```bash
docker run -d \
  -v /path/to/config.yaml:/app/config.yaml \
  -v /path/to/ssh:/root/.ssh \
  --name bazzite-updater \
  ghcr.io/brettmayson/bazzite-updater:latest
```

## Configuration

An example `config.yaml` file:

```yaml
machines:
  - ssh: updater@bazzite.local
    mac: aa:bb:cc:dd:ee:ff
    cron: 0 10 * * * *
    steam: true
    flatpak: true
```

It is recommended to use a dedicated user for the updater. The user will need to be able to run `sudo shutdown now` if you want the machine to turn off after updates.

### SSH

The updater uses SSH to connect, authorized keys are required.

### Wake on LAN

If a MAC address is provided, the updater will send a magic packet to wake the machine up before updating, if it is off.

### Cron

The updater will run on the schedule provided.

### Steam

If `steam` is set to `true`, the updater will monitor the steam client for network traffic, allowing it to complete game updates before shutting down. After 5 minutes of inactivity, the updater will shut the machine down (configurable with steam_delay).

### Flatpak

If `flatpak` is set to `true`, the updater will update all flatpak applications before shutting down. The user will need to be able to run `sudo flatpak update -y` without a password.
