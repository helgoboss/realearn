use crate::infrastructure::ui::{
    bindings::root, constants, HeaderPanel, MappingRowsPanel, SharedMainState,
};

use lazycell::LazyCell;
use reaper_high::Reaper;

use slog::debug;
use std::cell::Cell;

use crate::application::{MappingModel, SessionUi, WeakSession};
use crate::core::when;
use crate::domain::MappingId;
use crate::infrastructure::plugin::{App, RealearnPluginParameters};
use rx_util::UnitEvent;
use std::rc::{Rc, Weak};
use std::sync;
use swell_ui::{Dimensions, Pixels, SharedView, View, ViewContext, Window};

/// The complete ReaLearn panel containing everything.
// TODO-low Maybe call this SessionPanel
#[derive(Debug)]
pub struct MainPanel {
    view: ViewContext,
    active_data: LazyCell<ActiveData>,
    dimensions: Cell<Option<Dimensions<Pixels>>>,
    state: SharedMainState,
    plugin_parameters: sync::Weak<RealearnPluginParameters>,
}

#[derive(Debug)]
struct ActiveData {
    session: WeakSession,
    header_panel: SharedView<HeaderPanel>,
    mapping_rows_panel: SharedView<MappingRowsPanel>,
}

impl MainPanel {
    pub fn new(plugin_parameters: sync::Weak<RealearnPluginParameters>) -> Self {
        Self {
            view: Default::default(),
            active_data: LazyCell::new(),
            dimensions: None.into(),
            state: Default::default(),
            plugin_parameters,
        }
    }

    pub fn notify_session_is_available(self: Rc<Self>, session: WeakSession) {
        // Finally, the session is available. First, save its reference and create sub panels.
        let active_data = ActiveData {
            session: session.clone(),
            header_panel: HeaderPanel::new(
                session.clone(),
                self.state.clone(),
                self.plugin_parameters.clone(),
            )
            .into(),
            mapping_rows_panel: MappingRowsPanel::new(
                session,
                Rc::downgrade(&self),
                self.state.clone(),
            )
            .into(),
        };
        self.active_data.fill(active_data).unwrap();
        // If the plug-in window is currently open, open the sub panels as well. Now we are talking!
        if let Some(window) = self.view.window() {
            self.open_sub_panels(window);
        }
    }

    pub fn dimensions(&self) -> Dimensions<Pixels> {
        self.dimensions
            .get()
            .unwrap_or_else(|| constants::MAIN_PANEL_DIMENSIONS.in_pixels())
    }

    pub fn open_with_resize(self: SharedView<Self>, parent_window: Window) {
        #[cfg(target_family = "windows")]
        {
            // On Windows, the first time opening the dialog window is just to determine the best
            // dimensions based on HiDPI settings.
            // TODO-low If we skip this, the dimensions would be saved. Wouldn't that be better?
            //  I guess if there are multiple screens, keeping this line is better. Then it's a
            //  matter of reopening the GUI to improve scaling. Test it!
            self.dimensions.replace(None);
        }
        self.open(parent_window)
    }

    pub fn force_scroll_to_mapping(&self, mapping_id: MappingId) {
        if let Some(data) = self.active_data.borrow() {
            data.mapping_rows_panel.force_scroll_to_mapping(mapping_id);
        }
    }

    pub fn edit_mapping(&self, mapping: *const MappingModel) {
        if let Some(data) = self.active_data.borrow() {
            data.mapping_rows_panel.edit_mapping(mapping);
        }
    }

    fn open_sub_panels(&self, window: Window) {
        if let Some(data) = self.active_data.borrow() {
            data.header_panel.clone().open(window);
            data.mapping_rows_panel.clone().open(window);
        }
    }

    fn invalidate_status_text(&self) {
        let state = self.state.borrow();
        self.view
            .require_control(root::ID_MAIN_PANEL_STATUS_TEXT)
            .set_text(state.status_msg.get_ref().as_str());
    }

    fn invalidate_version_text(&self) {
        self.view
            .require_control(root::ID_MAIN_PANEL_VERSION_TEXT)
            .set_text(format!("ReaLearn {}", App::detailed_version_label()));
    }

    fn invalidate_all_controls(&self) {
        self.invalidate_version_text();
        self.invalidate_status_text();
    }

    fn register_listeners(self: SharedView<Self>) {
        let state = self.state.borrow();
        self.when(state.status_msg.changed(), |view| {
            view.invalidate_status_text();
        });
    }

    fn when(
        self: &SharedView<Self>,
        event: impl UnitEvent,
        reaction: impl Fn(SharedView<Self>) + 'static + Copy,
    ) {
        when(event.take_until(self.view.closed()))
            .with(Rc::downgrade(self))
            .do_async(move |panel, _| reaction(panel));
    }
}

impl View for MainPanel {
    fn dialog_resource_id(&self) -> u32 {
        root::ID_MAIN_PANEL
    }

    fn view_context(&self) -> &ViewContext {
        &self.view
    }

    fn opened(self: SharedView<Self>, window: Window) -> bool {
        #[cfg(target_family = "windows")]
        if self.dimensions.get().is_none() {
            // The dialog has been opened by user request but the optimal dimensions have not yet
            // been figured out. Figure them out now.
            self.dimensions.replace(Some(
                window.convert_to_pixels(constants::MAIN_PANEL_DIMENSIONS),
            ));
            // Close and reopen window, this time with `dimensions()` returning the optimal size to
            // the host.
            let parent_window = window.parent().expect("must have parent");
            window.destroy();
            self.open(parent_window);
            return false;
        }
        // Optimal dimensions have been calculated and window has been reopened. Now add sub panels!
        self.open_sub_panels(window);
        self.invalidate_all_controls();
        self.register_listeners();
        true
    }
}

impl SessionUi for Weak<MainPanel> {
    fn show_mapping(&self, mapping: *const MappingModel) {
        self.upgrade()
            .expect("main panel not existing anymore")
            .edit_mapping(mapping);
    }
}

impl Drop for MainPanel {
    fn drop(&mut self) {
        debug!(Reaper::get().logger(), "Dropping main panel...");
    }
}
