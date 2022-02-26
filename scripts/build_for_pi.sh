cargo build \
    --target arm-unknown-linux-gnueabi


# when built from x86-64 platform
# this required installing:
# "sudo apt install gcc-arm-linux-gnueabi"
# and adding:
# "openssl = { version = "0.10.29", features = ["vendored"] }"
# to toml as paho mqtt requires libssl-dev to be installed