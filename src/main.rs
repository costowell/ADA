mod drink;

use drink::DrinkApi;
use log::{error, info};

fn main() {
    dotenv::dotenv().unwrap();
    env_logger::init();
    
    let drink = DrinkApi::new(std::env::var("DRINK_API_KEY").unwrap());
    info!("Getting drink credits...");
    match drink.get_credits("cole") {
        Ok(user) => {
            info!("{} ({}) has {} drink credits", user.cn, user.uid, user.drink_balance);

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
