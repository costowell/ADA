#!/usr/bin/env bash

# Exit on error
set -e

# Might be prompted to reenter the password if some commands
# take too long to execute. Just run the whole thing as root.
if [ "$(id -u)" -ne 0 ]; then
    echo "Must be run as root."
    exit
fi

# Network is always a problem. Lets let them know early on.
if ! ping -c 5 8.8.8.8; then
    echo "No internet connection. Exiting..."
    exit
fi

# Update
apt -y update
apt -y upgrade

# Install necessary dependencies
apt -y install git cage foot libx11-dev libx11-xcb-dev xwayland libwayland-dev \
    libwayland-bin libwayland-egl1-mesa wayland-protocols libegl-mesa0 libegl1 \
    libfontconfig1-dev libfreefare-dev libfreetype6-dev libnfc-dev libssl-dev

# Enable GPIO UART pins (0 means enable apparently)
raspi-config nonint do_serial_hw 0

# Disable UART console (1 means disable... why?)
raspi-config nonint do_serial_cons 1

# Increase swap size because Rust takes lots of RAM to compile
dphys-swapfile swapoff
sed -i '/^CONF_SWAPSIZE=/s/=.*/=2024/' /etc/dphys-swapfile
dphys-swapfile setup
dphys-swapfile swapon
