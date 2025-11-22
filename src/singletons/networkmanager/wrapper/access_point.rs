#![allow(dead_code)]
use crate::singletons::networkmanager::enums::*;

#[derive(Debug, Clone)]
pub struct NetworkManagerAccessPoint {
    pub flags: u32, // a bitmap, see enums
    pub wpa_flags: u32, // a bitmap, see enums
    pub rsn_flags: u32, // a bitmap, see enums
    pub ssid: String,
    pub frequency: u32, // in MHz
    pub bssid: String, // hw address
    pub max_bitrate: u32, // in Kbps
    pub bandwidth: u32, // in MHz
    pub strength: u8, // 0-100
    pub last_seen: u64, // in seconds
}

impl NetworkManagerAccessPoint {
    pub fn flags_has_bit(&self, bit: NetworkManager80211ApFlags) -> bool {
        (self.flags & (bit as u32)) != 0
    }

    pub fn wpa_flags_has_bit(&self, bit: NetworkManager80211ApSecurityFlags) -> bool {
        (self.wpa_flags & (bit as u32)) != 0
    }

    pub fn rsn_flags_has_bit(&self, bit: NetworkManager80211ApSecurityFlags) -> bool {
        (self.rsn_flags & (bit as u32)) != 0
    }
}