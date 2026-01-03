//! # Trait Editor Plugin
//!
//! This plugin provides a professional multi-panel editor for creating trait definitions.
//! It supports .trait files (folder-based) that contain trait metadata and methods.
//!
//! ## File Types
//!
//! - **Trait Definition** (.trait folder)
//!   - Contains `trait.json` with the trait definition
//!   - Appears as a single file in the file drawer
//!
//! ## Editors
//!
//! - **Trait Editor**: Multi-panel editor with properties, methods, and code preview

use plugin_editor_api::*;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use gpui::*;
use ui::dock::PanelView;

// Trait Editor modules
mod editor;
mod method_editor;
mod workspace_panels;

// Re-export main types
pub use editor::TraitEditor;
pub use method_editor::{MethodEditorView, MethodEditorEvent};
pub use workspace_panels::{PropertiesPanel, MethodsPanel, CodePreviewPanel};

/// Storage for editor instances owned by the plugin
struct EditorStorage {
    panel: Arc<dyn ui::dock::PanelView>,
    wrapper: Box<TraitEditorWrapper>,
}

/// The Trait Editor Plugin
pub struct TraitEditorPlugin {
    editors: Arc<Mutex<HashMap<usize, EditorStorage>>>,
    next_editor_id: Arc<Mutex<usize>>,
}

impl Default for TraitEditorPlugin {
    fn default() -> Self {
        Self {
            editors: Arc::new(Mutex::new(HashMap::new())),
            next_editor_id: Arc::new(Mutex::new(0)),
        }
    }
}

impl EditorPlugin for TraitEditorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: PluginId::new("com.pulsar.trait-editor"),
            name: "Trait Editor".into(),
            version: "0.1.0".into(),
            author: "Pulsar Team".into(),
            description: "Professional multi-panel editor for creating trait definitions".into(),
        }
    }

    fn file_types(&self) -> Vec<FileTypeDefinition> {
        vec![
            FileTypeDefinition {
                id: FileTypeId::new("trait"),
                extension: "trait".to_string(),
                display_name: "Trait Definition".to_string(),
                icon: ui::IconName::Code,
                color: gpui::rgb(0x3F51B5).into(),
                structure: FileStructure::FolderBased {
                    marker_file: "trait.json".to_string(),
                    template_structure: vec![],
                },
                default_content: json!({
                    "name": "NewTrait",
                    "methods": []
                }),
                categories: vec!["Types".to_string()],
            }
        ]
    }

    fn editors(&self) -> Vec<EditorMetadata> {
        vec![EditorMetadata {
            id: EditorId::new("trait-editor"),
            display_name: "Trait Editor".into(),
            supported_file_types: vec![FileTypeId::new("trait")],
        }]
    }

    fn create_editor(
        &self,
        editor_id: EditorId,
        file_path: PathBuf,
        window: &mut Window,
        cx: &mut App,
        logger: &plugin_editor_api::EditorLogger,
    ) -> Result<(Arc<dyn PanelView>, Box<dyn EditorInstance>), PluginError> {
        logger.info("TRAIT EDITOR LOADED!!");
        if editor_id.as_str() == "trait-editor" {
            let actual_path = if file_path.is_dir() {
                file_path.join("trait.json")
            } else {
                file_path.clone()
            };

            let panel = cx.new(|cx| TraitEditor::new_with_file(actual_path.clone(), window, cx));
            let panel_arc: Arc<dyn ui::dock::PanelView> = Arc::new(panel.clone());
            let wrapper = Box::new(TraitEditorWrapper {
                panel: panel.into(),
                file_path: file_path.clone(),
            });

            let id = {
                let mut next_id = self.next_editor_id.lock().unwrap();
                let id = *next_id;
                *next_id += 1;
                id
            };

            self.editors.lock().unwrap().insert(id, EditorStorage {
                panel: panel_arc.clone(),
                wrapper: wrapper.clone(),
            });

            log::info!("Created trait editor instance {} for {:?}", id, file_path);
            Ok((panel_arc, wrapper))
        } else {
            Err(PluginError::EditorNotFound { editor_id })
        }
    }

    fn on_load(&mut self) {
        log::info!("Trait Editor Plugin loaded");
    }

    fn on_unload(&mut self) {
        let mut editors = self.editors.lock().unwrap();
        let count = editors.len();
        editors.clear();
        log::info!("Trait Editor Plugin unloaded (cleaned up {} editors)", count);
    }
}

#[derive(Clone)]
pub struct TraitEditorWrapper {
    panel: Entity<TraitEditor>,
    file_path: std::path::PathBuf,
}

impl plugin_editor_api::EditorInstance for TraitEditorWrapper {
    fn file_path(&self) -> &std::path::PathBuf {
        &self.file_path
    }

    fn save(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_save(window, cx)
        })
    }

    fn reload(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_reload(window, cx)
        })
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

export_plugin!(TraitEditorPlugin);
