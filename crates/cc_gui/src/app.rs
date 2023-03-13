use crate::pages::{auth_page, main_page};
use crate::state::{Route, State, Status};

pub struct App {
    state: State,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = State::new(&cc.egui_ctx);
        let mut this = Self { state };

        if this.state.oss.is_some() {
            this.state.get_list();
        }

        this
    }
}

impl eframe::App for App {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.state.setting.store();
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.state.init(ctx);
        match &mut self.state.status {
            Status::Idle(ref mut route) => match route {
                Route::Auth => auth_page(ctx, &mut self.state),
                Route::List => main_page(ctx, &mut self.state, frame),
                _ => {}
            },
            Status::Busy(ref mut route) => match route {
                Route::Auth => {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.spinner();
                        });
                    });
                }
                Route::List => main_page(ctx, &mut self.state, frame),
                _ => {}
            },
        };
    }
}
