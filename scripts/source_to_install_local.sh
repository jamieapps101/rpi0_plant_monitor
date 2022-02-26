echo "build package"
cargo build --release

echo "stop service if already installed"
sudo systemctl stop rpi0_plant_monitor.service

echo "install binary"
sudo cp ./target/release/rpi0_plant_monitor /usr/bin/rpi0_plant_monitor

echo "setup systemd scripts"
sudo cp ./scripts/rpi0_plant_monitor.service /etc/systemd/system/

echo "enable and begin systemd service"
sudo systemctl enable --now rpi0_plant_monitor.service