#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod btle;

use std::thread;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Sender, Receiver};

use btleplug::api::BDAddr;
use eframe::{egui, Frame};
use egui::{Context, vec2};
use std::default::Default;
use std::thread::JoinHandle;
use eframe::epaint::Color32;
use egui::RichText;

#[derive(Debug, Clone)]
pub enum BluetoothError {
    AdapterNotFound,
    FailedToScan,
    NoPeripherals,
    UnknownError
}

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        ..Default::default()
    };

    let (tx, rx) = tokio::sync::oneshot::channel();

    // Start a tokio thread
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("bluetooth-scanner")
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let runtime = Arc::new(runtime);
    let runtime_two = runtime.clone();

    // The tokio thread must be in a thread because runtime.block_on is a blocking operation
    thread::spawn(move || {
        // Wait until the receiver on our thread gets a message, then the block finishes and it is dropped
        runtime.block_on(async move {
            rx.await
        });
    });

    let _guard = runtime_two.enter();

    // TODO: UI only updates when it gets an input event, is there a fix?
    // Ex, when btle.rs sends an event, the window will only update with new Receive data
    // if I click on it or mouse over it or some other input event egui listens for
    //
    // eframe::App trait below also might have an option to fix this, like some sort of
    // "request_redraw" method or something similar
    eframe::run_native(
        "Btle Sniffer",
        options,
        Box::new(|_cc| Box::new(BTLESniffer::new()))
    );

    // Send a message to the spawned thread, causing it to finish and drop
    let _ = tx.send(());
    return Ok(())
}

struct BTLESniffer {
    receiver: Option<Receiver<Vec<BDAddr>>>,
    error_state: Option<BluetoothError>,
    addresses: Option<Vec<BDAddr>>
}

impl BTLESniffer {
    pub fn new() -> Self {
        return Self{
            receiver: None,
            error_state: None,
            addresses: None
        }
    }
}

impl eframe::App for BTLESniffer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // TODO: Set the spacing here - ui.set_style() <- this needs some style struct
            ui.heading("Bluetooth Low Energy Sniffer");
            ui.horizontal(|ui| {
                ui.label("Scan for Bluetooth devices: ");
                if ui.add(egui::Button::new("Scan")).clicked() {
                    let (tx, rx): (Sender<Vec<BDAddr>>, Receiver<Vec<BDAddr>>) = mpsc::channel();
                    self.receiver = Some(rx);
                    let _handle = tokio::task::spawn(btle::bluetooth_listener(tx, 10));
                }
            });
            // Check if our receiver exists and has info, update addresses state
            if let Some(receiver) = &self.receiver {
                if let Ok(addresses) = receiver.try_recv() {
                    self.addresses = Some(addresses);
                }
            }
            egui::Grid::new("btle-data-grid")
                .striped(true)
                .spacing(vec2(50f32, 0f32))
                .show(ui, |ui| {
                    ui.label("MAC Address");
                    ui.label("Debug");
                    ui.end_row();
                    match &self.addresses {
                        Some(addresses) => {
                            for address in addresses.iter() {
                                // TODO: Allow this to be copy-able
                                ui.label(RichText::new(format!("{}", address)));
                                ui.label("Empty");
                                ui.end_row();
                            }
                        }
                        None => ()
                    }
                });
        });
    }
}
