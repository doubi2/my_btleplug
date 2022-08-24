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

use super::{ble::watcher::BLEWatcher, peripheral::Peripheral, peripheral::PeripheralId};
use crate::{
    api::{BDAddr, Central, CentralEvent, ScanFilter},
    common::adapter_manager::AdapterManager,
    Error, Result,
};
use async_trait::async_trait;
use futures::stream::Stream;
use std::convert::TryInto;
use std::fmt::{self, Debug, Formatter};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use windows::Devices::Bluetooth::BluetoothLEDevice;
use windows::Devices::Enumeration::DeviceInformation;
use windows::Devices::Radios::{Radio, RadioKind};

/// Implementation of [api::Central](crate::api::Central).
#[derive(Clone)]
pub struct Adapter {
    watcher: Arc<Mutex<BLEWatcher>>,
    manager: Arc<AdapterManager<Peripheral>>,
}

impl Adapter {
    pub(crate) fn new() -> Self {
        let watcher = Arc::new(Mutex::new(BLEWatcher::new()));
        let manager = Arc::new(AdapterManager::default());
        Adapter { watcher, manager }
    }

    pub async fn get_all_BLEDevice() -> Result<Adapter>{
        let result: Adapter = Adapter::new();
        let radios = Radio::GetRadiosAsync().unwrap().await?;
        let radios = radios.into_iter().find(|x| x.Kind().unwrap() == RadioKind::Bluetooth);
        if radios.is_some(){
            let device_selector = BluetoothLEDevice::GetDeviceSelector()?;
            let device_collection = DeviceInformation::FindAllAsyncAqsFilter(&device_selector)?.
            get().expect("FindAllAsyncAqsFilter failed");

            for device_info in device_collection.into_iter() {
                let device_name = match device_info.Name() {
                    Ok(name) => name.to_string(),
                    Err(_) => "".to_string(),
                };
                if !device_name.contains("Inateck"){
                    continue
                }
                if let Ok(device_id) = device_info.Id() {
                    let ble_device = BluetoothLEDevice::FromIdAsync(device_id).unwrap().await.unwrap();
                    result.set_ble_device(ble_device,device_name)?;
                }
            }
        }
        Ok(result)
    }


    pub(crate) fn set_ble_device(&self, ble_device: BluetoothLEDevice,device_name:String) ->  Result<()>  {
        let manager = self.manager.clone();
        let bluetooth_address = ble_device.BluetoothAddress().unwrap();
        let address: BDAddr = bluetooth_address.try_into().unwrap();
        if let Some(mut entry) = manager.peripheral_mut(&address.into()) {
            if !device_name.is_empty() {
                entry.value_mut().update_local_name(device_name);
            }
            manager.emit(CentralEvent::DeviceUpdated(address.into()));
        } else {
            let peripheral = Peripheral::new(Arc::downgrade(&manager), address);
            if !device_name.is_empty() {
                peripheral.update_local_name(device_name);
            }
            manager.add_peripheral(peripheral);
            manager.emit(CentralEvent::DeviceDiscovered(address.into()));
        }
        Ok(())
    }
}

impl Debug for Adapter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Adapter")
            .field("manager", &self.manager)
            .finish()
    }
}

#[async_trait]
impl Central for Adapter {
    type Peripheral = Peripheral;

    async fn events(&self) -> Result<Pin<Box<dyn Stream<Item = CentralEvent> + Send>>> {
        Ok(self.manager.event_stream())
    }

    async fn start_scan(&self, _filter: ScanFilter) -> Result<()> {
        // TODO: implement filter
        let watcher = self.watcher.lock().unwrap();
        let manager = self.manager.clone();
        watcher.start(Box::new(move |args| {
            let bluetooth_address = args.BluetoothAddress().unwrap();
            let address: BDAddr = bluetooth_address.try_into().unwrap();
            //  F7:8C:81:74:B0:2A
            let address: BDAddr = [0xF7, 0x8C, 0x81, 0x74, 0xB0, 0x2A].into();
            if let Some(mut entry) = manager.peripheral_mut(&address.into()) {
                entry.value_mut().update_properties(args);
                manager.emit(CentralEvent::DeviceUpdated(address.into()));
            } else {
                let peripheral = Peripheral::new(Arc::downgrade(&manager), address);
                peripheral.update_properties(args);
                manager.add_peripheral(peripheral);
                manager.emit(CentralEvent::DeviceDiscovered(address.into()));
            }
        }))
    }

    async fn stop_scan(&self) -> Result<()> {
        let watcher = self.watcher.lock().unwrap();
        watcher.stop().unwrap();
        Ok(())
    }

    async fn peripherals(&self) -> Result<Vec<Peripheral>> {
        Ok(self.manager.peripherals())
    }

    async fn peripheral(&self, id: &PeripheralId) -> Result<Peripheral> {
        self.manager.peripheral(id).ok_or(Error::DeviceNotFound)
    }

    async fn add_peripheral(&self, _address: BDAddr) -> Result<Peripheral> {
        Err(Error::NotSupported(
            "Can't add a Peripheral from a BDAddr".to_string(),
        ))
    }

    async fn adapter_info(&self) -> Result<String> {
        // TODO: Get information about the adapter.
        Ok("WinRT".to_string())
    }
}
