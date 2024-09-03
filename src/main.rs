mod controller;
mod drink;
mod runtime;
mod ui;

use std::{thread::sleep, time::Duration};

use clap::{Arg, Command};
use controller::{codec::AdaControllerCommand, AdaController};
use drink::DrinkApi;
use log::{error, info};
use slint::ComponentHandle;
use tokio::sync::mpsc;
use runtime::runtime;

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

async fn stuff(rfid_device: String, controller_device: &str) {
    // Connect to controller and wait for it to initialize
    let controller = AdaController::new(controller_device);
    controller.listen().await.unwrap();
    sleep(Duration::from_secs(5));

    // Testing BS
    controller
        .send_command(controller::codec::AdaControllerCommand::Accept)
        .await;
    let mut rx = controller.rx.lock().await.clone().unwrap();
    println!("Maybe back?");
    loop {
        if rx.changed().await.is_err() {
            break;
        }
        if let Some(value) = rx.borrow_and_update().clone() {
            println!("{value}");
        }
    }
}

fn gatekeeper_listen(rfid_device: String) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel(10);

    std::thread::spawn(move || {
        //let mut member_listener =
        //    GateKeeperMemberListener::new(rfid_device, RealmType::Drink).unwrap();
        //loop {
        //    if let Some(association) = member_listener.poll_for_user() {
        //        if let Ok(user) = member_listener.fetch_user(association.clone()) {
        //            futures::executor::block_on(
        //                tx.send(user["user"]["uid"].as_str().unwrap().to_string()),
        //            )
        //            .unwrap();
        //        } else {
        //            error!("Failed to fetch user");
        //        }
        //    }
        //}

        std::thread::sleep(Duration::from_secs(2));

        futures::executor::block_on(tx.send("cole".to_string())).unwrap();
    });
    rx
}

fn main() {
    dotenvy::dotenv().unwrap();
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
    let rfid_device = matches
        .get_one::<String>("RFID_DEVICE")
        .unwrap()
        .to_string();
    let controller_device = matches
        .get_one::<String>("CONTROLLER_DEVICE")
        .unwrap()
        .to_string();

    let mut gatekeeper_rx = gatekeeper_listen(rfid_device);

    let window = ui::AppWindow::new().unwrap();
    let drink = DrinkApi::new(std::env::var("DRINK_API_KEY").unwrap());

    {
        let window = window.as_weak();
        slint::spawn_local(async move {
            let window = window.upgrade().unwrap();
            while let Some(uid) = gatekeeper_rx.recv().await {
                info!("Logged in as '{uid}'");
                match drink.get_credits(&uid) {
                    Ok(user) => window.login(user).await,
                    Err(err) => error!("Failed to get drink credits: {err}"),
                }
            }
        })
        .unwrap();
    }
    {
        let window = window.as_weak();
        std::thread::spawn(move || {
            let rt = runtime();
            rt.block_on(async {
                // Connect to controller and wait for it to initialize
                let controller = AdaController::new(&controller_device);
                controller.listen().await.unwrap();
                sleep(Duration::from_secs(5));

                // Accept bills
                controller.send_command(AdaControllerCommand::Accept).await;
                let mut rx = controller.rx.lock().await.clone().unwrap();

                loop {
                    if rx.changed().await.is_err() {
                        break;
                    }
                    if let Some(value) = rx.borrow_and_update().clone() {
                        window
                            .upgrade_in_event_loop(move |window| {
                                println!("{value}");
                                window
                                    .set_credits(window.get_credits() + value.to_credits() as i32);
                            })
                            .unwrap();
                    }
                }
            });
        });
    }

    window.run().unwrap();
}
