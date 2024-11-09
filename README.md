# Auto Drink Admin

Auto Drink Admin is an ATM for the Computer Science House's networked drink machines allowing members to get drink credits without having to find a drink admin.

This is the software which drives this fantastical machine.

## Setting Up

I have provided a few useful scripts to setup the a Raspberry Pi 3 B+ in `scripts/`.

Before you run these scripts, you must first provide credentials via a `.env` file. Luckily, there is a `.env.template` provided which should give you an idea of what is required.

Once you've done that, run these scripts.

> Note: You can run these scripts from anywhere! They will set everything up relative to the repository's location.
> As long as you don't move the repository's or the scripts' location, the system will be set up correctly.

``` sh
# Setup the system (update and install pkgs, adjust swap, config devices, etc)
sudo scripts/system_setup.sh

# Setup the user (install rust, compile program, setup autostart, etc)
scripts/user_setup.sh
```

Once they've executed successfully, reboot the system, and everything should be in order.
