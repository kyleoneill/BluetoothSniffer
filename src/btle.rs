use btleplug::api::{bleuuid::uuid_from_u16, BDAddr, Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::error::Error;
use std::thread;
use std::time::Duration;
use tokio;
use std::sync::mpsc::Sender;

use crate::BluetoothError;

pub async fn bluetooth_listener(sender: Sender<Vec<BDAddr>>, wait_time: u64) -> Result<(), BluetoothError> {
    let manager = Manager::new().await.unwrap();

    // Get bluetooth adapters
    let adapters = match manager.adapters().await {
        Ok(adapter) => adapter,
        Err(_) => return Err(BluetoothError::AdapterNotFound)
    };

    // Get the first adapter
    let adapter = match adapters.into_iter().nth(0) {
        Some(central) => central,
        None => return Err(BluetoothError::AdapterNotFound)
    };

    // start scanning for devices
    match adapter.start_scan(ScanFilter::default()).await {
        Ok(_) => (),
        Err(_) => return Err(BluetoothError::FailedToScan)
    };

    // Wait for items to be scanned
    tokio::time::sleep(Duration::from_secs(wait_time)).await;

    let peripherals = match adapter.peripherals().await {
        Ok(per) => per,
        Err(_) => return Err(BluetoothError::UnknownError)
    };

    if peripherals.is_empty() {
        return Err(BluetoothError::NoPeripherals)
    }
    else {
        let mut addresses: Vec<BDAddr> = Vec::new();
        for peripheral in peripherals.iter() {
            let address = peripheral.address();
            addresses.push(address);
        }
        println!("{:?}", addresses);
        sender.send(addresses).expect("Failed to send event");
    }

    return Ok(())

    // // TODO: Event stream rather than running it once with a wait
    // // Fetch our adapters event stream
    // let mut events = central.events().await?;
    // while let Some(event) = events.next().await {
    //     match event {
    //         CentralEvent::DeviceDiscovered(id) => {
    //
    //             sender.send()
    //             println!("DeviceDiscovered: {:?}", id);
    //         }
    //         _ => {}
    //     }
    // }

    // // instead of waiting, you can use central.events() to get a stream which will
    // // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    // time::sleep(Duration::from_secs(2)).await;
    //
}
