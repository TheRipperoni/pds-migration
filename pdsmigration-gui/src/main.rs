#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod agent;
mod create_account_page;
mod errors;
mod existing_pds_page;
mod home_page;
mod new_pds_page;
mod styles;

use crate::create_account_page::CreateAccountPage;
use crate::errors::GuiError;
use crate::existing_pds_page::ExistingPdsPage;
use crate::home_page::HomePage;
use crate::new_pds_page::NewPdsPage;
use eframe::egui;
use egui::Window;
use std::fmt::format;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use tokio::runtime::Runtime;

enum Page {
    Home(HomePage),
    OldLogin(ExistingPdsPage),
    NewLogin(NewPdsPage),
    CreateAccount(CreateAccountPage),
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        " PDS Migration Tool",
        options,
        Box::new(|_cc| Ok(Box::<PdsMigrationApp>::default())),
    )
}

struct PdsMigrationApp {
    page: Page,
    page_rx: Receiver<Page>,
    error_rx: Receiver<GuiError>,
    error_windows: Vec<ErrorWindow>,
}

impl Default for PdsMigrationApp {
    fn default() -> Self {
        let (page_tx, page_rx) = std::sync::mpsc::channel();
        let (error_tx, error_rx) = std::sync::mpsc::channel();

        Self {
            page: Page::OldLogin(ExistingPdsPage::new(page_tx, error_tx)),
            page_rx,
            error_rx,
            error_windows: vec![],
        }
    }
}

impl eframe::App for PdsMigrationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_page_update();
        self.check_for_errors(ctx);

        for error_window in &mut self.error_windows {
            error_window.show(ctx);
        }

        let styled_frame = styles::get_styled_frame();
        egui::CentralPanel::default()
            .frame(styled_frame)
            .show(ctx, |ui| {
                styles::set_text_color(ui);
                styles::render_title(ui, styles::FRAME_TITLE);

                match &mut self.page {
                    Page::Home(home_page) => {
                        home_page.show(ui);
                    }
                    Page::OldLogin(existing_pds_page) => {
                        existing_pds_page.show(ui);
                    }
                    Page::NewLogin(new_pds_page) => {
                        new_pds_page.show(ui);
                    }
                    Page::CreateAccount(create_account_page) => {
                        create_account_page.show(ui);
                    }
                }
            });
    }
}

impl PdsMigrationApp {
    pub fn check_page_update(&mut self) {
        let res = self.page_rx.try_recv();
        if res.is_ok() {
            self.page = res.unwrap();
        }
    }

    pub fn check_for_errors(&mut self, ctx: &egui::Context) {
        if let Ok(error) = self.error_rx.try_recv() {
            let error_window = ErrorWindow::new(error);
            self.error_windows.push(error_window);
        }
    }
}

pub struct ErrorWindow {
    open: bool,
    gui_error: GuiError,
}

impl ErrorWindow {
    pub fn new(gui_error: GuiError) -> Self {
        Self {
            open: true,
            gui_error,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        Window::new(self.gui_error.to_string())
            .open(&mut self.open.clone())
            .vscroll(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("{}", self.gui_error));
                let btn = ui.button("Ok");
                if btn.clicked() {
                    self.open = false;
                }
            });
    }
}
