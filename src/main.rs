#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod btle;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use btleplug::api::Peripheral;
use btleplug::platform::{Peripheral as BluetoothPeripheral, PeripheralId};
use eframe::{Frame, egui};
use egui::RichText;
use egui::vec2;
use std::default::Default;
use tokio::time;

use crate::btle::BluetoothListener;

#[derive(Debug, Clone)]
pub enum BluetoothError {
    AdapterNotFound,
    FailedToScan,
    NoPeripherals,
    UnknownError,
}

fn main() -> eframe::Result {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Btle Sniffer",
        options,
        Box::new(|cc| {
            let app = SnifferApp::new(cc);
            Ok(Box::new(app))
        }),
    )
}

struct SnifferApp {
    receiver: Receiver<BluetoothPeripheral>,
    error_state: Option<BluetoothError>,
    peripherals: HashMap<PeripheralId, BluetoothPeripheral>,
}

impl SnifferApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<BluetoothPeripheral>();
        std::thread::spawn(move || {
            // We need to build a Tokio runtime to use. egui must control the main thread so we cannot use the handy tokio::main macro to generate a runtime for us
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .thread_name("bluetooth-scanner")
                .enable_all()
                .enable_time()
                .build()
                .unwrap();
            runtime.block_on(async move {
                let bluetooth_listener = BluetoothListener::new(tx, 10).await.unwrap();
                let mut interval = time::interval(Duration::from_secs(1));
                // The first tick completes instantly
                interval.tick().await;
                loop {
                    interval.tick().await;
                    // TODO: This can return an error and I should do something about that
                    // with setting things on error_state
                    let _res = bluetooth_listener.listen().await;
                }
            })
        });
        Self {
            receiver: rx,
            error_state: None,
            peripherals: HashMap::new(),
        }
    }
    fn read_for_peripherals(&mut self) {
        while let Ok(peripheral) = self.receiver.try_recv() {
            let _res = self.peripherals.insert(peripheral.id(), peripheral);
        }
    }
}

impl eframe::App for SnifferApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        self.read_for_peripherals();

        // TODO: DISPLAY ERROR IF error_state IS SOME
        egui::CentralPanel::default().show(ui, |ui| {
            // TODO: Set the spacing here - ui.set_style() <- this needs some style struct
            ui.heading("Bluetooth Low Energy Sniffer");
            egui::Grid::new("btle-data-grid")
                .striped(true)
                .spacing(vec2(50f32, 0f32))
                .show(ui, |ui| {
                    ui.label("MAC Address");
                    ui.label("Debug");
                    ui.end_row();
                    for peripheral in self.peripherals.values() {
                        ui.label(RichText::new(peripheral.to_string()));
                        ui.label("Empty");
                        ui.end_row();
                    }
                });
        });
    }
}
