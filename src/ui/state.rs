use crate::modmanager::SyncEvent;

pub struct UiState {
    pub log: Vec<SyncEvent>,
    pub show_full_ui: bool,
    pub finished: bool,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            log: Vec::new(),
            show_full_ui: false,
            finished: false,
        }
    }
}
