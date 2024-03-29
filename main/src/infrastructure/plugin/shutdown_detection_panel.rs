use crate::infrastructure::plugin::BackboneShell;
use crate::infrastructure::ui::bindings::root;
use swell_ui::{SharedView, View, ViewContext, Window};

#[derive(Debug, Default)]
pub struct ShutdownDetectionPanel {
    view: ViewContext,
}

impl ShutdownDetectionPanel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl View for ShutdownDetectionPanel {
    fn dialog_resource_id(&self) -> u32 {
        root::ID_HIDDEN_PANEL
    }

    fn view_context(&self) -> &ViewContext {
        &self.view
    }

    fn show_window_on_init(&self) -> bool {
        false
    }

    fn on_destroy(self: SharedView<Self>, _window: Window) {
        BackboneShell::get().shutdown();
    }
}
