use std::{collections::HashMap, net::IpAddr, time::SystemTime};

use log::{debug, info};
use rust_async_tuyapi::{
    error::ErrorKind, mesparse::Message, tuyadevice::TuyaDevice, Payload, PayloadStruct,
};
use serde_json::json;

pub struct Outlet {
    pub dev_id: String,
    pub key: String,
    pub address: IpAddr,
}

impl Outlet {
    pub async fn get(&self) -> Result<bool, ErrorKind> {
        let mut device = self.device()?;
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        let dps = json!({"1": {"command": {"gwId": "", "devId": "", "uid": "", "t": ""}}, "7": {"command": {"devId": "", "uid": "", "t": ""}}, "8": {"command": {"gwId": "", "devId": ""}}, "9": {"command": {"gwId": "", "devId": ""}}, "10": {"command": {"gwId": "", "devId": "", "uid": "", "t": ""}}, "13": {"command": {"devId": "", "uid": "", "t": ""}}, "16": {"command": {"devId": "", "uid": "", "t": ""}}, "18": {"command": {"dpId": [18, 19, 20]}}, "64": {"command": {"reqType": "", "data": {}}}});

        let payload = Payload::Struct(PayloadStruct {
            dev_id: self.dev_id.to_string(),
            gw_id: None,
            uid: None,
            t: Some(current_time.to_string()),
            dp_id: None,
            dps: Some(dps),
        });

        debug!("connecting...");
        let mut receiver = device.connect().await?;
        info!("connected.");
        device.get(payload).await?;
        debug!("payload sent");

        match receiver.recv().await {
            Some(Ok(msgs)) => Self::parse_state_messages(&msgs).ok_or(ErrorKind::ParsingIncomplete),
            Some(Err(e)) => Err(e),
            None => Err(ErrorKind::TcpStreamClosed),
        }
    }

    fn parse_state_message(msg: &Message) -> Option<bool> {
        match &msg.payload {
            Payload::String(s) => {
                let data: HashMap<String, serde_json::Value> = serde_json::from_str(s).ok()?;
                let dps = data.get("dps")?;
                let dps: HashMap<String, serde_json::Value> =
                    serde_json::from_value(dps.clone()).ok()?;
                match dps.get("1") {
                    Some(serde_json::Value::Bool(b)) => Some(*b),
                    _ => None,
                }
            }
            _ => {
                debug!("no payload string in message, ignoring");
                None
            }
        }
    }

    fn parse_state_messages(msgs: &[Message]) -> Option<bool> {
        debug!("parsing received messages, there are {} msgs", msgs.len());
        if msgs.is_empty() {
            debug!("no message in stream");
            return None;
        }

        for msg in msgs {
            debug!("message is: {:?}", msg);
            if let Some(v) = Self::parse_state_message(msg) {
                return Some(v);
            }
        }

        debug!("no response had a boolean! weird.");
        None
    }

    pub async fn set(&self, state: bool) -> Result<(), ErrorKind> {
        let mut device = self.device()?;

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        let dps: HashMap<String, _> = [("1".to_string(), json!(state))].iter().cloned().collect();
        let dps = serde_json::to_value(&dps).expect("invalid json");

        let payload = rust_async_tuyapi::Payload::Struct(PayloadStruct {
            dev_id: self.dev_id.to_string(),
            gw_id: Some(self.dev_id.to_string()),
            uid: None,
            t: Some(current_time.to_string()),
            dp_id: None,
            dps: Some(dps),
        });

        device.connect().await?;
        device.set(payload).await
    }

    pub async fn toggle(&self) -> Result<(), ErrorKind> {
        let current = self.get().await?;
        self.set(!current).await
    }

    fn device(&self) -> Result<TuyaDevice, ErrorKind> {
        TuyaDevice::new("3.3", &self.dev_id, Some(&self.key), self.address)
    }
}
