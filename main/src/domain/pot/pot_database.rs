use crate::base::{blocking_read_lock, blocking_write_lock};
use crate::domain::pot::provider_database::{
    Database, DatabaseId, InnerFilterItem, ProviderContext, FIL_IS_AVAILABLE_FALSE,
    FIL_IS_AVAILABLE_TRUE, FIL_IS_FAVORITE_FALSE, FIL_IS_FAVORITE_TRUE, FIL_IS_SUPPORTED_FALSE,
    FIL_IS_SUPPORTED_TRUE, FIL_IS_USER_PRESET_FALSE, FIL_IS_USER_PRESET_TRUE,
    FIL_PRODUCT_KIND_EFFECT, FIL_PRODUCT_KIND_INSTRUMENT, FIL_PRODUCT_KIND_LOOP,
    FIL_PRODUCT_KIND_ONE_SHOT,
};
use crate::domain::pot::providers::directory::{DirectoryDatabase, DirectoryDbConfig};
use crate::domain::pot::providers::komplete::KompleteDatabase;
use crate::domain::pot::{
    BuildInput, Fil, FilterItem, FilterItemCollections, FilterItemId, InnerBuildInput,
    PersistentDatabaseId, PersistentPresetId, PluginId, Preset, PresetId, Stats,
};

use crate::domain::pot::plugins::PluginDatabase;
use crate::domain::pot::providers::defaults::DefaultsDatabase;
use crate::domain::pot::providers::ini::IniDatabase;
use crate::domain::AnyThreadBackboneState;
use enumset::{enum_set, EnumSet};
use indexmap::IndexSet;
use realearn_api::persistence::PotFilterKind;
use reaper_high::Reaper;
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub fn pot_db() -> &'static PotDatabase {
    use once_cell::sync::Lazy;
    static POT_DB: Lazy<PotDatabase> = Lazy::new(PotDatabase::open);
    &POT_DB
}

type BoxedDatabase = Box<dyn Database + Send + Sync>;
type DatabaseOpeningResult = Result<BoxedDatabase, PotDatabaseError>;

// By having the RwLocks around the provider databases and not around the pot database, we
// can have "more" concurrent access. E.g. find_preset_by_id, a function which can be called very
// often by the GUI, only read-locks one particular database, not all. So if another database
// is currently written to, it doesn't matter. It's just more flexible and also simplifies
// usage of PotDatabase from a consumer perspective, because we can easily obtain a static reference
// to it. Also, in future we might want to use some fork-join approach to refresh/search
// concurrently multiple databases. This would require having a RwLock around the database itself
// because we would need to pass the database reference to the async code with an Arc, but an Arc
// alone doesn't allow mutation of its contents. That's true even if the async database access would
// be read-only. The synchronous refresh would still need mutable access but we wouldn't be able to
// get one directly within an Arc.

pub struct PotDatabase {
    plugin_db: RwLock<PluginDatabase>,
    databases: BTreeMap<DatabaseId, RwLock<DatabaseOpeningResult>>,
    revision: AtomicU8,
}

#[derive(Clone, Debug, derive_more::Display)]
pub struct PotDatabaseError(String);

impl Error for PotDatabaseError {}

fn box_db<D: Database + Send + Sync + 'static>(
    opening_result: Result<D, Box<dyn Error>>,
) -> DatabaseOpeningResult {
    let db = opening_result.map_err(|e| PotDatabaseError(e.to_string()))?;
    Ok(Box::new(db))
}

impl PotDatabase {
    pub fn open() -> Self {
        let resource_path = Reaper::get().resource_path();
        let komplete_db = KompleteDatabase::open();
        let rfx_chain_db = {
            let config = DirectoryDbConfig {
                persistent_id: PersistentDatabaseId::new("fx-chains".to_string()),
                root_dir: resource_path.join("FXChains"),
                valid_extensions: &["RfxChain"],
                name: "FX chains",
                description: "All the RfxChain files in your FXChains directory",
            };
            DirectoryDatabase::open(config)
        };
        let track_template_db = {
            let config = DirectoryDbConfig {
                persistent_id: PersistentDatabaseId::new("track-templates".to_string()),
                root_dir: resource_path.join("TrackTemplates"),
                valid_extensions: &["RTrackTemplate"],
                name: "Track templates",
                description: "All the RTrackTemplate files in your TrackTemplates directory.\n\
                Doesn't load the complete track, only its FX chain!",
            };
            DirectoryDatabase::open(config)
        };
        let ini_db = IniDatabase::open(
            PersistentDatabaseId::new("fx-presets".to_string()),
            resource_path.join("presets"),
        );
        let defaults_db = DefaultsDatabase::open();
        let databases = [
            box_db(komplete_db),
            box_db(rfx_chain_db),
            box_db(track_template_db),
            box_db(ini_db),
            box_db(Ok(defaults_db)),
        ];
        let databases = databases
            .into_iter()
            .enumerate()
            .map(|(i, db)| (DatabaseId(i as _), RwLock::new(db)))
            .collect();
        let pot_database = Self {
            plugin_db: Default::default(),
            databases,
            revision: Default::default(),
        };
        pot_database.refresh();
        pot_database
    }

    /// Returns a number that will be increased with each database refresh.
    pub fn revision(&self) -> u8 {
        self.revision.load(Ordering::Relaxed)
    }

    pub fn refresh(&self) {
        // Build provider context
        let resource_path = Reaper::get().resource_path();
        let plugin_db = PluginDatabase::crawl(&resource_path);
        let provider_context = ProviderContext::new(&plugin_db);
        // Refresh databases
        for db in self.databases.values() {
            let mut db = blocking_write_lock(db, "pot db refresh provider db");
            let Some(db) = db.as_mut().ok() else {
                continue;
            };
            let _ = db.refresh(&provider_context);
        }
        // Memorize plug-ins
        *blocking_write_lock(&self.plugin_db, "pot db refresh plugin db") = plugin_db;
        // Increment revision
        self.revision.fetch_add(1, Ordering::Relaxed);
    }

    pub fn build_collections(&self, mut input: BuildInput) -> BuildOutput {
        // Preparation
        let any_thread_backbone_state = AnyThreadBackboneState::get();
        let favorites = blocking_read_lock(
            &any_thread_backbone_state.pot_favorites,
            "favorites build collections",
        );
        let plugin_db = blocking_read_lock(&self.plugin_db, "pot db build collections 0");
        let provider_context = ProviderContext::new(&plugin_db);
        // Build constant filter collections
        let mut total_output = BuildOutput {
            supported_filter_kinds: enum_set!(
                PotFilterKind::Database
                    | PotFilterKind::IsUser
                    | PotFilterKind::IsFavorite
                    | PotFilterKind::ProductKind
            ),
            ..Default::default()
        };
        measure_duration(&mut total_output.stats.filter_query_duration, || {
            add_constant_filter_items(&mut input, &mut total_output.filter_item_collections);
            // Let all databases build filter collections and accumulate them
            let mut database_filter_items = Vec::new();
            let mut used_product_ids = HashSet::new();
            for (db_id, db) in &self.databases {
                // If the database is on the exclude list, we don't even want it to appear in the
                // database list.
                if input.filter_excludes.excludes_database(*db_id) {
                    continue;
                }
                // Acquire database access
                let db = blocking_read_lock(db, "pot db build_collections 1");
                let Ok(db) = db.as_ref() else {
                    continue
                };
                // Create database filter item
                let filter_item = FilterItem {
                    persistent_id: "".to_string(),
                    id: FilterItemId(Some(Fil::Database(*db_id))),
                    parent_name: None,
                    name: Some(db.name().to_string()),
                    icon: None,
                    more_info: Some(db.description().to_string()),
                };
                database_filter_items.push(filter_item);
                // Don't continue if database doesn't match filter
                // (but it should appear on the list)
                if !input.filters.database_matches(*db_id) {
                    continue;
                }
                // Add supported filter kinds
                total_output.supported_filter_kinds |= db.supported_advanced_filter_kinds();
                // Build and accumulate filters collections
                let inner_input = InnerBuildInput::new(&input, &favorites, *db_id);
                let Ok(filter_collections) = db.query_filter_collections(&provider_context, inner_input) else {
                    continue;
                };
                // Add unique filter items directly to the list of filters. Gather shared filter
                // items so we can deduplicate them later.
                for (kind, items) in filter_collections.into_iter() {
                    let final_filter_items = items.into_iter().filter_map(|i| match i {
                        InnerFilterItem::Unique(i) => Some(i),
                        InnerFilterItem::Product(pid) => {
                            used_product_ids.insert(pid);
                            None
                        }
                    });
                    total_output
                        .filter_item_collections
                        .extend(kind, final_filter_items);
                }
            }
            // Process shared filter items
            let product_filter_items = used_product_ids.into_iter().filter_map(|pid| {
                let product = plugin_db.find_product_by_id(&pid)?;
                let filter_item = FilterItem {
                    persistent_id: "".to_string(),
                    id: FilterItemId(Some(Fil::Product(pid))),
                    parent_name: None,
                    name: Some(product.name.clone()),
                    icon: None,
                    more_info: product.kind.map(|k| k.to_string()),
                };
                Some(filter_item)
            });
            total_output
                .filter_item_collections
                .extend(PotFilterKind::Bank, product_filter_items);
            // Add database filter items
            if input.affected_kinds.contains(PotFilterKind::Database) {
                total_output
                    .filter_item_collections
                    .set(PotFilterKind::Database, database_filter_items);
            }
            // Important: At this point, some previously selected filters might not exist anymore.
            // So we should reset them and not let them influence the preset query anymore!
            input.filters.clear_if_not_available_anymore(
                input.affected_kinds,
                &total_output.filter_item_collections,
            );
        });
        // Finally build and accumulate presets
        let mut sortable_preset_ids: Vec<_> =
            measure_duration(&mut total_output.stats.preset_query_duration, || {
                self.databases
                    .iter()
                    .filter(|(db_id, _)| {
                        input.filters.database_matches(**db_id)
                            && !input.filter_excludes.excludes_database(**db_id)
                    })
                    .filter_map(|(db_id, db)| {
                        // Acquire database access
                        let db = blocking_read_lock(db, "pot db build_collections 2");
                        let db = db.as_ref().ok()?;
                        // Let database build presets
                        let inner_input = InnerBuildInput::new(&input, &favorites, *db_id);
                        let preset_ids = db.query_presets(&provider_context, inner_input).ok()?;
                        Some((*db_id, preset_ids))
                    })
                    .flat_map(|(db_id, preset_ids)| preset_ids.into_iter().map(move |p| (db_id, p)))
                    .collect()
            });
        // Sort filter items and presets
        measure_duration(&mut total_output.stats.sort_duration, || {
            for (kind, collection) in total_output.filter_item_collections.iter_mut() {
                if kind.wants_sorting() {
                    collection.sort_by(|i1, i2| {
                        lexical_sort::lexical_cmp(i1.sort_name(), i2.sort_name())
                    });
                }
            }
            sortable_preset_ids.sort_by(|(_, p1), (_, p2)| {
                lexical_sort::lexical_cmp(&p1.preset_name, &p2.preset_name)
            });
        });
        // Index presets. Because later, we look up the preset index by the preset ID and vice versa
        // and we want that to happen without complexity O(n)! There can be tons of presets!
        measure_duration(&mut total_output.stats.index_duration, || {
            total_output.preset_collection = sortable_preset_ids
                .into_iter()
                .map(|(db_id, p)| PresetId::new(db_id, p.inner_preset_id))
                .collect();
        });
        total_output
    }

    pub fn find_preset_by_id(&self, preset_id: PresetId) -> Option<Preset> {
        let plugin_db = blocking_read_lock(&self.plugin_db, "pot db find_preset_by_id 0");
        let provider_context = ProviderContext::new(&plugin_db);
        let db = self.databases.get(&preset_id.database_id)?;
        let db = blocking_read_lock(db, "pot db find_preset_by_id 1");
        let db = db.as_ref().ok()?;
        db.find_preset_by_id(&provider_context, preset_id.preset_id)
    }

    pub fn try_with_plugin_db<R>(
        &self,
        f: impl FnOnce(&PluginDatabase) -> R,
    ) -> Result<R, &'static str> {
        let plugin_db = self
            .plugin_db
            .try_read()
            .map_err(|_| "couldn't acquire plugin db lock")?;
        let r = f(&plugin_db);
        Ok(r)
    }

    pub fn try_with_db<R>(
        &self,
        db_id: DatabaseId,
        f: impl FnOnce(&dyn Database) -> R,
    ) -> Result<R, &'static str> {
        let db = self.databases.get(&db_id).ok_or("database not found")?;
        let db = db
            .try_read()
            .map_err(|_| "couldn't acquire provider db lock")?;
        let db = db.as_ref().map_err(|_| "provider database not opened")?;
        let r = f(&**db);
        Ok(r)
    }

    pub fn try_find_preset_by_id(
        &self,
        preset_id: PresetId,
    ) -> Result<Option<Preset>, &'static str> {
        let plugin_db = self
            .plugin_db
            .try_read()
            .map_err(|_| "couldn't acquire plugin db lock")?;
        let provider_context = ProviderContext::new(&plugin_db);
        let db = self
            .databases
            .get(&preset_id.database_id)
            .ok_or("database not found")?;
        let db = db
            .try_read()
            .map_err(|_| "couldn't acquire provider db lock")?;
        let db = db.as_ref().map_err(|_| "provider database not opened")?;
        Ok(db.find_preset_by_id(&provider_context, preset_id.preset_id))
    }

    /// Ignores exclude lists.
    pub fn find_unsupported_preset_matching(
        &self,
        plugin_id: &PluginId,
        preset_name: &str,
    ) -> Option<PersistentPresetId> {
        let product_id = {
            let plugin_db = blocking_read_lock(
                &self.plugin_db,
                "plugin db find_unsupported_preset_matching",
            );
            let plugin = plugin_db.find_plugin_by_id(plugin_id)?;
            plugin.common.core.product_id
        };
        self.databases.values().find_map(|db| {
            // Acquire database access
            let db = blocking_read_lock(db, "pot db find_unsupported_preset_matching");
            let db = db.as_ref().ok()?;
            // Find preset
            let preset = db.find_unsupported_preset_matching(product_id, preset_name)?;
            Some(preset.common.persistent_id.clone())
        })
    }
}

#[derive(Default)]
pub struct BuildOutput {
    pub supported_filter_kinds: EnumSet<PotFilterKind>,
    pub filter_item_collections: FilterItemCollections,
    pub preset_collection: IndexSet<PresetId>,
    pub stats: Stats,
}

fn measure_duration<R>(duration: &mut Duration, f: impl FnOnce() -> R) -> R {
    let start = Instant::now();
    let r = f();
    *duration = start.elapsed();
    r
}

fn add_constant_filter_items(
    input: &mut BuildInput,
    filter_item_collections: &mut FilterItemCollections,
) {
    if input.affected_kinds.contains(PotFilterKind::IsAvailable) {
        filter_item_collections.set(
            PotFilterKind::IsAvailable,
            create_filter_items_is_available(),
        );
    }
    if input.affected_kinds.contains(PotFilterKind::IsSupported) {
        filter_item_collections.set(
            PotFilterKind::IsSupported,
            create_filter_items_is_supported(),
        );
    }
    if input.affected_kinds.contains(PotFilterKind::IsFavorite) {
        filter_item_collections.set(PotFilterKind::IsFavorite, create_filter_items_is_favorite());
    }
    if input.affected_kinds.contains(PotFilterKind::IsUser) {
        filter_item_collections.set(PotFilterKind::IsUser, create_filter_items_is_user());
    }
    if input.affected_kinds.contains(PotFilterKind::ProductKind) {
        filter_item_collections.set(
            PotFilterKind::ProductKind,
            create_filter_items_product_kind(),
        );
    }
}

fn create_filter_items_is_available() -> Vec<FilterItem> {
    vec![
        FilterItem::simple(FIL_IS_AVAILABLE_FALSE, "Not available", '❌', ""),
        FilterItem::simple(
            FIL_IS_AVAILABLE_TRUE,
            "Available",
            '✔',
            "Usually means that the \
        corresponding plug-in has been scanned before by REAPER.\n\
        For Komplete, it means that the preset file itself is available.",
        ),
    ]
}

fn create_filter_items_is_supported() -> Vec<FilterItem> {
    vec![
        FilterItem::simple(FIL_IS_SUPPORTED_FALSE, "Not supported", '☹', ""),
        FilterItem::simple(
            FIL_IS_SUPPORTED_TRUE,
            "Supported",
            '☺',
            "Means that Pot Browser \
        can automatically load the preset into the corresponding plug-in.",
        ),
    ]
}

fn create_filter_items_is_favorite() -> Vec<FilterItem> {
    vec![
        FilterItem::simple(FIL_IS_FAVORITE_FALSE, "Not favorite", '☆', ""),
        FilterItem::simple(FIL_IS_FAVORITE_TRUE, "Favorite", '★', ""),
    ]
}

fn create_filter_items_product_kind() -> Vec<FilterItem> {
    vec![
        FilterItem::none(),
        FilterItem::simple(FIL_PRODUCT_KIND_INSTRUMENT, "Instrument", '🎹', ""),
        FilterItem::simple(FIL_PRODUCT_KIND_EFFECT, "Effect", '✨', ""),
        FilterItem::simple(FIL_PRODUCT_KIND_LOOP, "Loop", '➿', ""),
        FilterItem::simple(FIL_PRODUCT_KIND_ONE_SHOT, "One shot", '💥', ""),
    ]
}

fn create_filter_items_is_user() -> Vec<FilterItem> {
    vec![
        FilterItem::simple(FIL_IS_USER_PRESET_FALSE, "Factory preset", '🏭', ""),
        FilterItem::simple(FIL_IS_USER_PRESET_TRUE, "User preset", '🕵', ""),
    ]
}
