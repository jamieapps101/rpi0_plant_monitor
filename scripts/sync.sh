echo "Synchronising source files"
rsync -a \
    --delete \
    --exclude target \
    `pwd` \
    pzero:/home/jamie/

echo "Building"
ssh pzero "cd /home/jamie/rpi0_plant_monitor; /home/jamie/.cargo/bin/cargo build"