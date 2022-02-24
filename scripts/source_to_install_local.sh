echo "build package"
cargo build --release

echo "stop service if already installed"
sudo systemctl stop bme280monitor.service

echo "install binary"
sudo cp ./target/release/bme280monitor /usr/bin/bme280monitor

echo "setup systemd scripts"
sudo cp ./bme280monitor.service /etc/systemd/system/

echo "enable and begin systemd service"
sudo systemctl enable --now bme280monitor.service