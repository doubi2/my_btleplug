// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.

use std::error::Error;
use btleplug::api::{BDAddr, bleuuid::uuid_from_u16, Central, CharPropFlags, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use btleplug::winrtble::{ble::device::BLEDevice, ble::characteristic::BLECharacteristic, utils};
use rand::{thread_rng, Rng};
use std::time::Duration;
use uuid::Uuid;
use tokio::time;
use windows::Devices::Bluetooth::{BluetoothConnectionStatus, BluetoothLEDevice};
use windows::Foundation::TypedEventHandler;

/// Only devices whose name contains this string will be tried.
const PERIPHERAL_NAME_MATCH_FILTER: &str = "Inateck";

const MAC_PATH: &str = "F7:8C:81:74:B0:2A";
/// UUID of the characteristic for which we should subscribe to notifications.
const READ_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x0000af0100001000800000805f9b34fb);
const WRITE_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x0000af0200001000800000805f9b34fb);
const WRITE_WITHOUT_RESPONSE_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x0000af0400001000800000805f9b34fb);
const NOTIFY_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x0000af0300001000800000805f9b34fb);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let adapter = Adapter::get_all_BLEDevice().await?;

    let peripherals = adapter.peripherals().await?;

    if peripherals.is_empty() {
        eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
    } else {
        // All peripheral devices in range.
        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await?;
            let is_connected = peripheral.is_connected().await?;

            let properties = properties.unwrap();
            let local_name = properties
                .local_name
                .unwrap_or(String::from("(peripheral name unknown)"));
            println!(
                "Peripheral {:?} is connected: {:?}",
                &local_name, is_connected
            );
            // Check if it's the peripheral we want.
            // if local_name.contains(PERIPHERAL_NAME_MATCH_FILTER) {
            println!("Found matching peripheral {:?}...", &local_name);
            if !is_connected {
                // Connect if we aren't already connected.
                if let Err(err) = peripheral.connect().await {
                    eprintln!("Error connecting to peripheral, skipping: {:?}", err);
                    continue;
                }
            }
            let is_connected = peripheral.is_connected().await?;
            println!(
                "Now connected ({:?}) to peripheral {:?}.",
                is_connected, &local_name
            );
            if is_connected {
                println!("Discover peripheral {:?} services...", local_name);
                peripheral.discover_services().await?;
                for characteristic in peripheral.characteristics() {
                    println!("Checking characteristic {:?}", characteristic);
                    if characteristic.uuid == READ_CHARACTERISTIC_UUID {
                        for _ in 0..100 {
                            let resp = peripheral.read(&characteristic).await?;
                            println!("resp:{:?}", resp);
                            time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                    // if characteristic.uuid == WRITE_WITHOUT_RESPONSE_CHARACTERISTIC_UUID {
                    //     let color_cmd = vec![0x01, 0x02, 0x03, 0xAA];
                    //     peripheral.write(&characteristic, &color_cmd, WriteType::WithoutResponse)
                    //         .await?;
                    //     time::sleep(Duration::from_millis(200)).await;
                    // }
                    // if characteristic.uuid == WRITE_CHARACTERISTIC_UUID {
                    //     let color_cmd = vec![0x02, 0x02, 0x03, 0xAA];
                    //     peripheral.write(&characteristic, &color_cmd, WriteType::WithResponse)
                    //         .await?;
                    //     time::sleep(Duration::from_millis(200)).await;
                    // }
                    // if characteristic.uuid == NOTIFY_CHARACTERISTIC_UUID
                    //     && characteristic.properties.contains(CharPropFlags::NOTIFY)
                    // {
                    //     println!("Subscribing to characteristic {:?}", characteristic.uuid);
                    //     peripheral.subscribe(&characteristic).await?;
                    //     // Print the first 4 notifications received.
                    //     let mut notification_stream =
                    //         peripheral.notifications().await?.take(4);
                    //     // Process while the BLE connection is not broken or stopped.
                    //     while let Some(data) = notification_stream.next().await {
                    //         println!(
                    //             "Received data from {:?} [{:?}]: {:?}",
                    //             local_name, data.uuid, data.value
                    //         );
                    //     }
                    // }
                }
                println!("Disconnecting from peripheral {:?}...", local_name);
                peripheral.disconnect().await?;
            }
        }
    }
    Ok(())
}
