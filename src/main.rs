mod controller;
mod drink;
mod runtime;
mod ui;

use std::{io::Cursor, sync::Arc, thread::sleep, time::Duration};

use clap::{Arg, Command};
use controller::{codec::AdaControllerCommand, AdaController};
use drink::DrinkApi;
use gatekeeper_members::{GateKeeperMemberListener, RealmType};
use image::{io::Reader as ImageReader, GenericImageView};
use log::{error, info};
use runtime::runtime;
use slint::{ComponentHandle, Image, SharedPixelBuffer};
use tokio::sync::{mpsc, watch, Mutex};

fn gatekeeper_listen(rfid_device: String) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel(10);

    std::thread::spawn(move || {
        let mut member_listener =
            GateKeeperMemberListener::new(rfid_device, RealmType::Drink).unwrap();
        loop {
            if let Some(association) = member_listener.poll_for_user() {
                if let Ok(user) = member_listener.fetch_user(association.clone()) {
                    futures::executor::block_on(
                        tx.send(user["user"]["uid"].as_str().unwrap().to_string()),
                    )
                    .unwrap();
                } else {
                    error!("Failed to fetch user");
                }
            }
        }

        //loop {
        //    std::thread::sleep(Duration::from_secs(2));
        //    futures::executor::block_on(tx.send("cole".to_string())).unwrap();
        //}
    });
    rx
}

// Math reference: https://math.stackexchange.com/questions/1649714/whats-the-equation-for-a-rectircle-perfect-rounded-corner-rectangle-without-s
// Keeping here in case we gotta do some manual image processing
fn is_in_rounded_box(x: u32, y: u32, width: u32, height: u32, r: f32) -> bool {
    let a = width as f32 / 2.0;
    let b = height as f32 / 2.0;
    let x = x as f32 - a;
    let y = y as f32 - b;
    let value = (x.abs() / a).powf(2.0 * a / r) + (y.abs() / b).powf(2.0 * b / r);
    value <= 1.0
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
    let session_duration = window.get_session_duration();
    let (uid_tx, mut uid_rx) = watch::channel(None);

    {
        let window = window.as_weak();
        let rt = runtime();
        rt.spawn(async move {
            while let Some(uid) = gatekeeper_rx.recv().await {
                uid_tx.send(Some(uid.clone())).unwrap();
                info!("Logged in as '{uid}'");
                match drink.get_credits(&uid).await {
                    Ok(user) => {
                        let window = window.clone();
                        let bytes =
                            reqwest::get(format!("https://profiles.csh.rit.edu/image/{}", uid))
                                .await
                                .unwrap()
                                .bytes()
                                .await
                                .unwrap();
                        let img = ImageReader::new(Cursor::new(bytes))
                            .with_guessed_format()
                            .unwrap()
                            .decode()
                            .unwrap();
                        let (width, height) = img.dimensions();

                        window
                            .upgrade_in_event_loop(move |window| {
                                window.login(
                                    user,
                                    Image::from_rgba8(SharedPixelBuffer::clone_from_slice(
                                        &img.to_rgba8(),
                                        width,
                                        height,
                                    )),
                                );
                            })
                            .unwrap();

                        tokio::time::sleep(Duration::from_millis(session_duration as u64)).await;
                        uid_tx.send(None).unwrap();
                        window
                            .upgrade_in_event_loop(move |window| window.logout())
                            .unwrap();

                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                    Err(err) => error!("Failed to get drink credits: {err}"),
                }
            }
        });
    }
    {
        let window = window.as_weak();
        std::thread::spawn(move || {
            let rt = runtime();
            rt.block_on(async {
                // Connect to controller and wait for it to initialize
                let controller = AdaController::new(&controller_device);
                controller.connect().await.unwrap();
                sleep(Duration::from_secs(5));
                let uid = Arc::new(Mutex::new(None));

                {
                    let uid = uid.clone();
                    let mut rx = controller.rx.lock().await.clone().unwrap();
                    tokio::spawn(async move {
                        loop {
                            if rx.changed().await.is_err() {
                                break;
                            }
                            if let Some(uid) = &*uid.lock().await {
                                if let Some(value) = rx.borrow_and_update().clone() {
                                    println!("Adding {} to {}'s balance", value.to_credits(), uid);
                                    window
                                        .upgrade_in_event_loop(move |window| {
                                            println!("{value}");
                                            window.set_credits(
                                                window.get_credits() + value.to_credits() as i32,
                                            );
                                        })
                                        .unwrap();
                                }
                            }
                        }
                    });
                }
                {
                    loop {
                        if uid_rx.changed().await.is_err() {
                            break;
                        }
                        if let Some(value) = uid_rx.borrow_and_update().clone() {
                            controller.send_command(AdaControllerCommand::Accept).await;
                            *uid.lock().await = Some(value);
                        } else {
                            controller.send_command(AdaControllerCommand::Inhibit).await;
                            *uid.lock().await = None;
                        }
                    }
                }
            });
        });
    }

    window.run().unwrap();
}
