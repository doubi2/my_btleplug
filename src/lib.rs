// btleplug Source Code File
//
// Copyright 2020 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.
//
// Some portions of this file are taken and/or modified from Rumble
// (https://github.com/mwylde/rumble), using a dual MIT/Apache License under the
// following copyright:
//
// Copyright (c) 2014 The Rust Project Developers

//! BtlePlug is a Bluetooth Low Energy (BLE) central module library for Rust. It
//! currently supports Windows 10, macOS (and possibly iOS), Linux (using BlueZ
//! sockets instead of D-Bus). Android support is coming in a future update.
//!
//! ## Usage
//!
//! An example of how to use the library to control some BLE smart lights:
//!
//! ```rust,no_run
//! use btleplug::api::async_api::{Central, Peripheral as _};
//! use btleplug::api::{bleuuid::uuid_from_u16, WriteType};
//! use btleplug::platform::{Adapter, Manager, Peripheral};
//! use rand::{Rng, thread_rng};
//! use std::error::Error;
//! use std::thread;
//! use std::time::Duration;
//! use tokio::time;
//! use uuid::Uuid;
//!
//! const LIGHT_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xFFE9);
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let manager = Manager::new().await.unwrap();
//!
//!     // get the first bluetooth adapter
//!     let adapters = manager.adapters().await?;
//!     let central = adapters.into_iter().nth(0).unwrap();
//!
//!     // start scanning for devices
//!     central.start_scan().await?;
//!     // instead of waiting, you can use central.event_receiver() to fetch a channel and
//!     // be notified of new devices
//!     time::sleep(Duration::from_secs(2)).await;
//!
//!     // find the device we're interested in
//!     let light = find_light(&central).await.unwrap();
//!
//!     // connect to the device
//!     light.connect().await?;
//!
//!     // discover characteristics
//!     light.discover_characteristics().await?;
//!
//!     // find the characteristic we want
//!     let chars = light.characteristics();
//!     let cmd_char = chars.iter().find(|c| c.uuid == LIGHT_CHARACTERISTIC_UUID).unwrap();
//!
//!     // dance party
//!     let mut rng = thread_rng();
//!     for _ in 0..20 {
//!         let color_cmd = vec![0x56, rng.gen(), rng.gen(), rng.gen(), 0x00, 0xF0, 0xAA];
//!         light.write(&cmd_char, &color_cmd, WriteType::WithoutResponse).await?;
//!         time::sleep(Duration::from_millis(200)).await;
//!     }
//!     Ok(())
//! }
//!
//! async fn find_light(central: &Adapter) -> Option<Peripheral> {
//!     for p in central.peripherals().await.unwrap() {
//!         if p.properties()
//!             .await
//!             .unwrap()
//!             .local_name
//!             .iter()
//!             .any(|name| name.contains("LEDBlue"))
//!         {
//!             return Some(p);
//!         }
//!     }
//!     None
//! }
//! ```

// We won't actually use anything specifically out of this crate. However, if we
// want the CoreBluetooth code to compile, we need the objc protocols
// (specifically, the core bluetooth protocols) exposed by it.
#[cfg(any(target_os = "macos", target_os = "ios"))]
extern crate cocoa;

use crate::api::ParseBDAddrError;
use std::result;
use std::time::Duration;

pub mod api;
#[cfg(all(target_os = "linux", feature = "async"))]
mod bluez;
mod common;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod corebluetooth;
pub mod platform;
#[cfg(target_os = "windows")]
mod winrtble;

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error("Permission denied")]
    PermissionDenied,

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Not connected")]
    NotConnected,

    #[error("The operation is not supported: {}", _0)]
    NotSupported(String),

    #[error("Timed out after {:?}", _0)]
    TimedOut(Duration),

    #[error("Error parsing UUID: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Invalid Bluetooth address: {0}")]
    InvalidBDAddr(#[from] ParseBDAddrError),

    #[error("{}", _0)]
    Other(String),
}

// BtlePlug Result type
pub type Result<T> = result::Result<T, Error>;
