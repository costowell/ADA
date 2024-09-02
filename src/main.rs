mod controller;
mod drink;

use std::{os::unix::thread, thread::sleep, time::Duration};

use clap::{Arg, Command};
use controller::AdaController;
use drink::DrinkApi;
use log::{error, info};

fn lmao() {
    let drink = DrinkApi::new(std::env::var("DRINK_API_KEY").unwrap());
    info!("Getting drink credits...");
    match drink.get_credits("keylime") {
        Ok(user) => {
            info!(
                "{} ({}) has {} drink credits",
                user.cn, user.uid, user.drink_balance
            );

            info!("Setting to {} drink credits...", user.drink_balance);
            match drink.set_credits(&user.uid, user.drink_balance) {
                Ok(()) => info!("Successfully set credits to {}!", user.drink_balance),
                Err(err) => error!("Failed to set drink credits: {}", err),
            }
        }
        Err(err) => {
            error!("Failed to get drink credits: {}", err);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    env_logger::init();

    let command = Command::new("ADA")
        .version("0.0.1")
        .author("Cole Stowell <cole@stowell.pro>")
        .about("Auto Drink Admin (probably should come up with a better name)")
        .arg(
            Arg::new("RFID_DEVICE")
                .help("Device connection string (e.g. pn532_uart:/dev/ttyUSB0)")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("CONTROLLER_DEVICE")
                .help("Device connection string (e.g. /dev/ttyACM0)")
                .required(true)
                .index(2),
        );

    let matches = command.get_matches();
    let rfid_device = matches.get_one::<String>("RFID_DEVICE").unwrap();
    let controller_device = matches.get_one::<String>("CONTROLLER_DEVICE").unwrap();

    // Connect to controller and wait for it to initialize
    let controller = AdaController::new(controller_device);
    controller.listen().await.unwrap();
    sleep(Duration::from_secs(5));

    // Testing BS
    controller
        .send_command(controller::codec::AdaControllerCommand::Accept)
        .await;
    let mut rx = controller.rx.lock().await.clone().unwrap();
    loop {
        if rx.changed().await.is_err() {
            break;
        }
        if let Some(value) = rx.borrow_and_update().clone() {
            println!("{value}");
        }
    }
}
