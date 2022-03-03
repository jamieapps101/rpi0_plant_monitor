#! /usr/bin env bash

echo "-> Build package"
cargo build
# cargo build --release

echo "-> Stop service if already installed"
sudo systemctl stop rpi0_plant_monitor.service

echo "-> Install config file"
sudo rm -rf /etc/plant_monitor
sudo mkdir /etc/plant_monitor
if test -f "./config/config.toml"; then
    sudo cp ./monitor/config/config.toml /etc/plant_monitor/config.toml
else
    echo "-> Could not find ./config/config.toml, using example file"
    sudo cp ./monitor/config/config.toml.example /etc/plant_monitor/config.toml
fi

echo "-> Install binary"
# sudo cp ./target/release/rpi0_plant_monitor /usr/bin/rpi0_plant_monitor
sudo cp ./target/debug/monitor /usr/bin/rpi0_plant_monitor

echo "-> Setup systemd scripts"
sudo cp ./scripts/rpi0_plant_monitor.service /etc/systemd/system/

echo "-> Enable and begin systemd service"
sudo systemctl enable --now rpi0_plant_monitor.service

echo "-> Complete"