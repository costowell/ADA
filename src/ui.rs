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
        self.set_offset(self.get_tick());
        self.set_error_message("".into());
    }
    pub fn logout(&self) {
        self.set_logged_in(false);
    }
}
