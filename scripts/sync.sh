rsync -a \
    --delete \
    --exclude target \
    `pwd` \
    192.168.0.22:/home/jamie/


ssh 192.168.0.22 "cd /home/jamie/rpi0_plant_monitor; /home/jamie/.cargo/bin/cargo build"