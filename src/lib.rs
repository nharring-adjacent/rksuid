#![feature(test)]

#[macro_use]
extern crate arrayref;

extern crate test;

pub mod ksuid {
    use base_encode::{from_str, to_string};
    use rand::distributions::Standard;
    use rand::prelude::*;
    extern crate time;
    use chrono::prelude::*;
    use time::Duration;

    #[derive(Debug, PartialOrd)]
    pub struct Ksuid {
        pub timestamp: u32,
        pub payload: u128,
    }

    impl PartialEq for Ksuid {
        fn eq(&self, other: &Self) -> bool {
            self.payload == other.payload && self.timestamp == other.timestamp
        }
    }
    impl Eq for Ksuid {}

    // Creates new ksuid with optionally specified timestamp and payload
    pub fn new(timestamp: Option<u32>, payload: Option<u128>) -> Ksuid {
        let internal_timestamp = match timestamp {
            None => gen_timestamp(),
            Some(i) => i,
        };
        let internal_payload = match payload {
            None => gen_payload(),
            Some(i) => i,
        };
        Ksuid {
            timestamp: internal_timestamp,
            payload: internal_payload,
        }
    }

    impl Ksuid {
        // Serialize ksuid into base62 encoded string
        pub fn serialize(&self) -> String {
            let alphabet = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
            let tstamp_be_bytes: Vec<u8> = self.timestamp.to_be_bytes().to_vec();
            let payload_be_bytes: Vec<u8> = self.payload.to_be_bytes().to_vec();
            let merged_bytes: Vec<u8> = tstamp_be_bytes
                .iter()
                .copied()
                .chain(payload_be_bytes.iter().copied())
                .collect();
            let mut merged_string =
                to_string(array_ref![merged_bytes, 0, 20], 62, alphabet).unwrap();
            if merged_string.char_indices().count() < 27 {
                // We will zero pad the left side of the string to get it to the required 27
                let num_zeros = 27 - merged_string.char_indices().count();
                let zero_str = String::from("0").repeat(num_zeros);
                merged_string = zero_str + merged_string.as_str();
            }
            return merged_string;
        }
    }

    // creates new ksuid from base62 encoded string serialized representation
    pub fn deserialize(text: &str) -> Ksuid {
        let alphabet = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let bytes_from_str_be_parsed = from_str(text, 62, alphabet);
        if let Some(bytes_from_str_be) = bytes_from_str_be_parsed {
            let timestamp_bytes: &[u8; 4] = array_ref![bytes_from_str_be, 0, 4];
            let payload_bytes: &[u8; 16] = array_ref![bytes_from_str_be, 4, 16];
            let timestamp: u32 = u32::from_be_bytes(*timestamp_bytes);
            let payload: u128 = u128::from_be_bytes(*payload_bytes);
            let ksuid = new(Some(timestamp), Some(payload));
            return ksuid;
        } else {
            panic!();
        }
    }

    // Returns a fresh random u128 for use as payload
    fn gen_payload() -> u128 {
        let payload: u128 = StdRng::from_entropy().sample(Standard);
        return payload;
    }
    // Returns now as u32 seconds since the unix epoch + 14e8 (May 13, 2014)
    fn gen_timestamp() -> u32 {
        let base_epoch = gen_epoch();
        let now = Utc::now();
        now.signed_duration_since(base_epoch).num_seconds() as u32
    }

    // Returns a Chrono::DateTime representing the adjusted epoch
    pub fn gen_epoch() -> DateTime<Utc> {
        Utc.timestamp(1400000000, 0)
    }

    pub fn to_std_epoch(timestamp: u32) -> DateTime<Utc> {
        let base_epoch = gen_epoch();
        base_epoch + Duration::seconds(timestamp as i64)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use rand::distributions::Standard;
    use rand::prelude::*;
    use std::{thread, time};
    use test::Bencher;

    #[test]
    fn test_new_with_timestamp() {
        let ksuid = ksuid::new(Some(85), None);
        assert_eq!(ksuid.timestamp, 85);
    }
    #[test]
    fn test_new() {
        let first = ksuid::new(None, None);
        thread::sleep(time::Duration::from_millis(2000));
        let second = ksuid::new(None, None);
        assert_ne!(first.timestamp, second.timestamp);
    }
    #[test]
    fn test_new_with_payload() {
        let payload: u128 = StdRng::from_entropy().sample(Standard);
        let ksuid = ksuid::new(None, Some(payload));
        assert_eq!(payload, ksuid.payload);
    }
    #[test]
    fn test_new_with_payload_and_timestamp() {
        let payload: u128 = StdRng::from_entropy().sample(Standard);
        let epoch_base = ksuid::gen_epoch();
        let timestamp = Utc::now().signed_duration_since(epoch_base).num_seconds() as u32;
        let ksuid = ksuid::new(Some(timestamp), Some(payload));
        assert_eq!(ksuid.payload, payload);
        assert_eq!(ksuid.timestamp, timestamp);
    }
    #[test]
    fn test_serialize_with_random_data_returns_right_length() {
        let ksuid = ksuid::new(None, None);
        let serialized = ksuid.serialize();
        println!(
            "Got ksuid: {:?} which serialized to: {:?}",
            ksuid, serialized
        );
        assert_eq!(serialized.char_indices().count(), 27);
    }
    #[test]
    fn test_serialize_deserialize() {
        let ksuid = ksuid::new(None, None);
        let serialized = ksuid.serialize();
        let ksuid2 = ksuid::deserialize(&serialized);
        assert_eq!(ksuid, ksuid2);
    }
    #[bench]
    fn bench_new_ksuid_creation(b: &mut Bencher) {
        b.iter(|| ksuid::new(None, None));
    }

    #[bench]
    fn bench_create_and_serialize(b: &mut Bencher) {
        b.iter(|| ksuid::new(None, None).serialize());
    }

    #[bench]
    fn bench_deserialize(b: &mut Bencher) {
        let ksuid = ksuid::new(None, None).serialize();
        b.iter(|| ksuid::deserialize(&ksuid));
    }
}