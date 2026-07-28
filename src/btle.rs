use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::BluetoothError;

pub struct BluetoothListener {
    sender: Sender<Peripheral>,
    manager: Manager,
    adapter: Adapter,
    wait_time: u64,
}

impl BluetoothListener {
    pub async fn new(sender: Sender<Peripheral>, wait_time: u64) -> Result<Self, BluetoothError> {
        let manager = Manager::new().await.unwrap();
        let adapters = match manager.adapters().await {
            Ok(adapter) => adapter,
            Err(_) => return Err(BluetoothError::AdapterNotFound),
        };
        let adapter = match adapters.into_iter().nth(0) {
            Some(central) => central,
            None => return Err(BluetoothError::AdapterNotFound),
        };
        Ok(Self {
            sender,
            manager,
            adapter,
            wait_time,
        })
    }
}

impl BluetoothListener {
    pub async fn listen(&self) -> Result<(), BluetoothError> {
        // start scanning for devices
        match self.adapter.start_scan(ScanFilter::default()).await {
            Ok(_) => (),
            Err(_) => return Err(BluetoothError::FailedToScan),
        };

        // TODO: Can use an event stream rather than a wait? This snippet was from old code so verify that it is useful
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

        // Wait for items to be scanned
        tokio::time::sleep(Duration::from_secs(self.wait_time)).await;

        let peripherals = match self.adapter.peripherals().await {
            Ok(per) => per,
            Err(_) => return Err(BluetoothError::UnknownError),
        };

        for peripheral in peripherals {
            let _res = self.sender.send(peripheral);
        }

        Ok(())
    }
}
