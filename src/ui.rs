use slint::Image;

use crate::drink::DrinkUser;

slint::include_modules!();

impl AppWindow {
    pub fn login(&self, user: DrinkUser, profile_picture: Image) {
        self.set_logged_in(true);
        self.set_uid(user.uid.clone().into());
        self.set_name(user.cn.clone().into());
        self.set_credits(user.drink_balance.try_into().unwrap());
        self.set_profile_picture(profile_picture);
        self.set_seconds_to_logout(self.get_max_seconds_to_logout());
    }
    pub fn logout(&self) {
        self.set_logged_in(false);
    }
}
