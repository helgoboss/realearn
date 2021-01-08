use crate::application::{RealearnControlSurface, WeakSession};
use crate::core::default_util::is_default;
use crate::core::Global;
use crate::domain::{RealearnControlSurfaceMainTask, RealearnControlSurfaceMiddleware};
use crate::infrastructure::data::{
    FileBasedControllerPresetManager, FileBasedMainPresetManager, FileBasedPresetLinkManager,
    SharedControllerPresetManager, SharedMainPresetManager, SharedPresetLinkManager,
};
use crate::infrastructure::plugin::debug_util;
use crate::infrastructure::server;
use crate::infrastructure::server::{RealearnServer, SharedRealearnServer, COMPANION_WEB_APP_URL};
use reaper_high::{Fx, MiddlewareControlSurface, Reaper};
use reaper_medium::RegistrationHandle;
use reaper_rx::{ActionRxHookPostCommand, ActionRxHookPostCommand2};
use rx_util::UnitEvent;
use rxrust::prelude::*;
use serde::{Deserialize, Serialize};
use slog::{debug, o, Drain};
use std::cell::{Ref, RefCell};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use url::Url;

make_available_globally_in_main_thread!(App);

pub struct App {
    controller_manager: SharedControllerPresetManager,
    main_preset_manager: SharedMainPresetManager,
    preset_link_manager: SharedPresetLinkManager,
    server: SharedRealearnServer,
    config: RefCell<AppConfig>,
    changed_subject: RefCell<LocalSubject<'static, (), ()>>,
    list_of_recently_focused_fx: Rc<RefCell<ListOfRecentlyFocusedFx>>,
    party_is_over_subject: LocalSubject<'static, (), ()>,
}

impl Default for App {
    fn default() -> Self {
        let config = AppConfig::load().unwrap_or_else(|e| {
            debug!(crate::application::App::logger(), "{}", e);
            Default::default()
        });
        App::new(config)
    }
}

#[derive(Default)]
struct ListOfRecentlyFocusedFx {
    previous: Option<Fx>,
    current: Option<Fx>,
}

impl ListOfRecentlyFocusedFx {
    fn feed(&mut self, currently_focused_fx: Option<Fx>) {
        self.previous = self.current.take();
        self.current = currently_focused_fx;
    }
}

impl App {
    pub fn detailed_version_label() -> &'static str {
        static DETAILED_VERSION: once_cell::sync::Lazy<String> =
            once_cell::sync::Lazy::new(build_detailed_version);
        &DETAILED_VERSION
    }

    fn new(config: AppConfig) -> App {
        App {
            controller_manager: Rc::new(RefCell::new(FileBasedControllerPresetManager::new(
                App::realearn_preset_dir_path().join("controller"),
            ))),
            main_preset_manager: Rc::new(RefCell::new(FileBasedMainPresetManager::new(
                App::realearn_preset_dir_path().join("main"),
            ))),
            preset_link_manager: Rc::new(RefCell::new(FileBasedPresetLinkManager::new(
                App::realearn_auto_load_configs_dir_path(),
            ))),
            server: Rc::new(RefCell::new(RealearnServer::new(
                config.main.server_http_port,
                config.main.server_https_port,
                App::server_resource_dir_path().join("certificates"),
                crate::application::App::get()
                    .control_surface_server_task_sender()
                    .clone(),
            ))),
            config: RefCell::new(config),
            changed_subject: Default::default(),
            list_of_recently_focused_fx: Default::default(),
            party_is_over_subject: Default::default(),
        }
    }

    /// Executed globally just once as soon as we have access to global REAPER instance
    pub fn init(&self) {
        crate::application::App::get().register_global_learn_action();
        server::keep_informing_clients_about_sessions();
        debug_util::register_resolve_symbols_action();
        crate::infrastructure::test::register_test_action();
        let list_of_recently_focused_fx = self.list_of_recently_focused_fx.clone();
        Global::control_surface_rx()
            .fx_focused()
            .take_until(self.party_is_over())
            .subscribe(move |fx| {
                list_of_recently_focused_fx.borrow_mut().feed(fx);
            });
    }

    pub fn wake_up(&self) -> RegistrationHandle<RealearnControlSurface> {
        if self.config.borrow().server_is_enabled() {
            self.server()
                .borrow_mut()
                .start()
                .unwrap_or_else(warn_about_failed_server_start);
        }
        let mut session = Reaper::get().medium_session();
        session
            .plugin_register_add_hook_post_command::<ActionRxHookPostCommand<Global>>()
            .unwrap();
        // This fails before REAPER 6.20 and therefore we don't have MIDI CC action feedback.
        let _ =
            session.plugin_register_add_hook_post_command_2::<ActionRxHookPostCommand2<Global>>();
        let surface = crate::application::App::get().take_control_surface();
        surface.middleware().reset();
        debug!(
            crate::application::App::logger(),
            "Registering ReaLearn control surface..."
        );
        session
            .plugin_register_add_csurf_inst(surface)
            .expect("couldn't register ReaLearn control surface")
    }

    pub fn go_to_sleep(&self, reg_handle: RegistrationHandle<RealearnControlSurface>) {
        let mut session = Reaper::get().medium_session();
        debug!(
            crate::application::App::logger(),
            "Unregistering ReaLearn control surface..."
        );
        unsafe {
            let surface = session
                .plugin_register_remove_csurf_inst(reg_handle)
                .expect("conrol surface was not registered");
            crate::application::App::get().put_control_surface_back(surface);
        }
        session.plugin_register_remove_hook_post_command_2::<ActionRxHookPostCommand2<Global>>();
        session.plugin_register_remove_hook_post_command::<ActionRxHookPostCommand<Global>>();
        self.server().borrow_mut().stop();
    }

    /// The special thing about this is that this doesn't return the currently focused FX but the
    /// last focused one. That's important because when queried from ReaLearn UI, the current one
    /// is mostly ReaLearn itself - which is in most cases not what we want.
    pub fn previously_focused_fx(&self) -> Option<Fx> {
        self.list_of_recently_focused_fx.borrow().previous.clone()
    }

    // TODO-medium Return a reference to a SharedControllerManager! Clients might just want to turn
    //  this into a weak one.
    pub fn controller_manager(&self) -> SharedControllerPresetManager {
        self.controller_manager.clone()
    }

    pub fn main_preset_manager(&self) -> SharedMainPresetManager {
        self.main_preset_manager.clone()
    }

    pub fn preset_link_manager(&self) -> SharedPresetLinkManager {
        self.preset_link_manager.clone()
    }

    pub fn server(&self) -> &SharedRealearnServer {
        &self.server
    }

    pub fn config(&self) -> Ref<AppConfig> {
        self.config.borrow()
    }

    pub fn start_server_persistently(&self) -> Result<(), String> {
        self.server.borrow_mut().start()?;
        self.change_config(AppConfig::enable_server);
        Ok(())
    }

    pub fn disable_server_persistently(&self) {
        self.change_config(AppConfig::disable_server);
    }

    pub fn enable_server_persistently(&self) {
        self.change_config(AppConfig::enable_server);
    }

    /// Logging debug info is always initiated by a particular session.
    pub fn log_debug_info(&self, session_id: &str) {
        self.server.borrow().log_debug_info(session_id);
        self.controller_manager.borrow().log_debug_info();
        // Must be the last because it (intentionally) panics
        crate::application::App::get().log_debug_info();
    }

    pub fn changed(&self) -> impl UnitEvent {
        self.changed_subject.borrow().clone()
    }

    fn change_config(&self, op: impl FnOnce(&mut AppConfig)) {
        let mut config = self.config.borrow_mut();
        op(&mut config);
        config.save().unwrap();
        self.notify_changed();
    }

    fn helgoboss_resource_dir_path() -> PathBuf {
        Reaper::get().resource_path().join("Helgoboss")
    }

    fn realearn_resource_dir_path() -> PathBuf {
        App::helgoboss_resource_dir_path().join("ReaLearn")
    }

    pub fn realearn_data_dir_path() -> PathBuf {
        Reaper::get()
            .resource_path()
            .join("Data/helgoboss/realearn")
    }

    pub fn realearn_preset_dir_path() -> PathBuf {
        Self::realearn_data_dir_path().join("presets")
    }

    pub fn realearn_auto_load_configs_dir_path() -> PathBuf {
        Self::realearn_data_dir_path().join("auto-load-configs")
    }

    fn server_resource_dir_path() -> PathBuf {
        Self::helgoboss_resource_dir_path().join("Server")
    }

    fn notify_changed(&self) {
        self.changed_subject.borrow_mut().next(());
    }

    fn party_is_over(&self) -> impl UnitEvent {
        self.party_is_over_subject.clone()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.party_is_over_subject.next(());
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    main: MainConfig,
}

impl AppConfig {
    pub fn load() -> Result<AppConfig, String> {
        let ini_content = fs::read_to_string(&Self::config_file_path())
            .map_err(|_| "couldn't read config file".to_string())?;
        let config = serde_ini::from_str(&ini_content).map_err(|e| format!("{:?}", e))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), &'static str> {
        let ini_content = serde_ini::to_string(self).map_err(|_| "couldn't serialize config")?;
        let config_file_path = Self::config_file_path();
        fs::create_dir_all(&config_file_path.parent().unwrap())
            .expect("couldn't create config directory");
        fs::write(config_file_path, ini_content).map_err(|_| "couldn't write config file")?;
        Ok(())
    }

    pub fn enable_server(&mut self) {
        self.main.server_enabled = 1;
    }

    pub fn disable_server(&mut self) {
        self.main.server_enabled = 0;
    }

    pub fn server_is_enabled(&self) -> bool {
        self.main.server_enabled > 0
    }

    pub fn companion_web_app_url(&self) -> url::Url {
        Url::parse(&self.main.companion_web_app_url).expect("invalid companion web app URL")
    }

    fn config_file_path() -> PathBuf {
        App::realearn_resource_dir_path().join("realearn.ini")
    }
}

#[derive(Serialize, Deserialize)]
struct MainConfig {
    #[serde(default, skip_serializing_if = "is_default")]
    server_enabled: u8,
    #[serde(
        default = "default_server_http_port",
        skip_serializing_if = "is_default_server_http_port"
    )]
    server_http_port: u16,
    #[serde(
        default = "default_server_https_port",
        skip_serializing_if = "is_default_server_https_port"
    )]
    server_https_port: u16,
    #[serde(
        default = "default_companion_web_app_url",
        skip_serializing_if = "is_default_companion_web_app_url"
    )]
    companion_web_app_url: String,
}

const DEFAULT_SERVER_HTTP_PORT: u16 = 39080;
const DEFAULT_SERVER_HTTPS_PORT: u16 = 39443;

fn default_server_http_port() -> u16 {
    DEFAULT_SERVER_HTTP_PORT
}

fn is_default_server_http_port(v: &u16) -> bool {
    *v == DEFAULT_SERVER_HTTP_PORT
}

fn default_server_https_port() -> u16 {
    DEFAULT_SERVER_HTTPS_PORT
}

fn is_default_server_https_port(v: &u16) -> bool {
    *v == DEFAULT_SERVER_HTTPS_PORT
}

fn default_companion_web_app_url() -> String {
    COMPANION_WEB_APP_URL.to_string()
}

fn is_default_companion_web_app_url(v: &str) -> bool {
    v == COMPANION_WEB_APP_URL
}

impl Default for MainConfig {
    fn default() -> Self {
        MainConfig {
            server_enabled: Default::default(),
            server_http_port: default_server_http_port(),
            server_https_port: default_server_https_port(),
            companion_web_app_url: default_companion_web_app_url(),
        }
    }
}

fn build_detailed_version() -> String {
    use crate::infrastructure::plugin::built_info::*;
    let dirty_mark = if GIT_DIRTY.contains(&true) {
        "-dirty"
    } else {
        ""
    };
    let date_info = if let Ok(d) = chrono::DateTime::parse_from_rfc2822(BUILT_TIME_UTC) {
        d.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        BUILT_TIME_UTC.to_string()
    };
    let debug_mark = if PROFILE == "debug" { "-debug" } else { "" };
    format!(
        "v{}/{}{} rev {}{} ({})",
        PKG_VERSION,
        CFG_TARGET_ARCH,
        debug_mark,
        GIT_COMMIT_HASH
            .map(|h| h[0..6].to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        dirty_mark,
        date_info
    )
}

pub fn warn_about_failed_server_start(info: String) {
    Reaper::get().show_console_msg(format!(
        "Couldn't start ReaLearn projection server because {}",
        info
    ))
}
