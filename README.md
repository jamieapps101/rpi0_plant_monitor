# Raspberry Pi Plant Monitor

Jamie Apps

---

This is a project written in Rust which tracks environmental plant data and can automatically water when required.

## Monitor
A set of functions to allow sending environmental and soil sensor data over MQTT to a Telegraf receiver using Influx data format. Can also receive IO control commands in JSON to control an LED and Pump pins. Various settings controllable through toml file. Program run using systemd service. Service file and install scripts also included.


## Controller
As yet unbuilt. Will run on containerised platform, and provide web GUI as well as server to automatically engage the water pump when soil mosture levels are low.