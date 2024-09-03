use std::{io::Read, str::FromStr};

use strum::{Display, EnumString};
use tokio_util::{
    bytes::BufMut,
    codec::{Decoder, Encoder},
};

pub struct AdaControllerCodec;

#[derive(Debug, EnumString, Clone, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AdaControllerResponse {
    // Coins
    Penny,
    Nickel,
    Dime,
    Quarter,

    // Bills
    OneDollar,
    FiveDollar,
    TenDollar,
    TwentyDollar,
}

#[derive(Debug, Display, Clone)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AdaControllerCommand {
    Inhibit,
    Accept,
}

impl AdaControllerResponse {
    pub fn to_credits(&self) -> u32 {
        match self {
            Self::Penny => 1,
            Self::Nickel => 5,
            Self::Dime => 10,
            Self::Quarter => 25,
            Self::OneDollar => 100,
            Self::FiveDollar => 500,
            Self::TenDollar => 1000,
            Self::TwentyDollar => 2000
        }
    }
}

impl Decoder for AdaControllerCodec {
    type Item = AdaControllerResponse;
    type Error = anyhow::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let end = src.as_ref().iter().position(|x| *x == b'\n');
        if let Some(end) = end {
            if let Some((_, command)) = src.split_to(end + 1).split_last() {
                let command = std::str::from_utf8(command)?;
                return Ok(Some(AdaControllerResponse::from_str(command)?));
            }
        }
        Ok(None)
    }
}

impl Encoder<AdaControllerCommand> for AdaControllerCodec {
    type Error = anyhow::Error;

    fn encode(
        &mut self,
        item: AdaControllerCommand,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let item = item.to_string();
        dst.reserve(item.len() + 1);
        dst.put_slice(item.as_bytes());
        dst.put_u8(b'\n');
        Ok(())
    }
}
