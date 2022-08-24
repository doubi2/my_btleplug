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

use super::adapter::Adapter;
use crate::{api, Result};
use async_trait::async_trait;
use windows::Devices::Bluetooth::BluetoothLEDevice;
use windows::Devices::Enumeration::DeviceInformation;
use windows::Devices::Radios::{Radio, RadioKind};
use crate::api::BDAddr;

/// Implementation of [api::Manager](crate::api::Manager).
#[derive(Clone, Debug)]
pub struct Manager {}

impl Manager {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    // pub async fn get_all_BLEDevice() -> Result<Adapter>{
    //     let result: Adapter = Adapter::new();
    //     let radios = Radio::GetRadiosAsync().unwrap().await?;
    //     let radios = radios.into_iter().find(|x| x.Kind().unwrap() == RadioKind::Bluetooth);
    //     if radios.is_some(){
    //         let device_selector = BluetoothLEDevice::GetDeviceSelector().unwrap();
    //         let device_collection = DeviceInformation::FindAllAsyncAqsFilter(&device_selector).
    //             unwrap().get().expect("FindAllAsyncAqsFilter failed");
    //
    //         for device_info in device_collection.into_iter() {
    //             let device_name = match device_info.Name() {
    //                 Ok(name) => name.to_string(),
    //                 Err(_) => "".to_string(),
    //             };
    //             if let Ok(device_id) = device_info.Id() {
    //                 let ble_device = BluetoothLEDevice::FromIdAsync(device_id).unwrap().await.unwrap();
    //                 result.set_ble_device(ble_device,device_name)?;
    //             }
    //         }
    //     }
    //     Ok(result)
    // }
}

#[async_trait]
impl api::Manager for Manager {
    type Adapter = Adapter;

    async fn adapters(&self) -> Result<Vec<Adapter>> {
        let mut result: Vec<Adapter> = vec![];
        let radios = Radio::GetRadiosAsync().unwrap().await.unwrap();

        for radio in &radios {
            let kind = radio.Kind().unwrap();
            if kind == RadioKind::Bluetooth {
                result.push(Adapter::new());
            }
        }
        return Ok(result);
    }
}
