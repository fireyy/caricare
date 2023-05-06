use crate::global;
use crate::state::{NavgatorType, State, Update};

pub fn location_bar_ui(ui: &mut egui::Ui, state: &mut State) {
    let response = ui.add_sized(
        ui.available_size(),
        egui::TextEdit::singleline(&mut state.current_path),
    );
    if response.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Enter))
        && state.current_path != state.navigator.location()
    {
        global()
            .update_tx
            .send(Update::Navgator(NavgatorType::New(
                state.current_path.clone(),
            )))
            .unwrap();
    }
}
