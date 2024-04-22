use eframe::egui::TextureId;

pub(crate) struct SpawnEditor {
    pub(crate) showing: bool,
    pub(crate) editor: Option<spawn_editor::frontend::Frontend>,
    map_texture_id: TextureId,
}

impl SpawnEditor {
    pub(crate) fn show(&mut self, path: &String, npc_format_fn: Box<dyn Fn(u32) -> String>) {
        let editor =
            spawn_editor::frontend::Frontend::init(path, self.map_texture_id, npc_format_fn)
                .unwrap();

        self.showing = true;
        self.editor = Some(editor);
    }

    pub(crate) fn update_spawn_path(
        &mut self,
        path: &str,
        npc_format_fn: Box<dyn Fn(u32) -> String>,
    ) {
        if self.editor.is_some() {
            self.editor = Some(
                spawn_editor::frontend::Frontend::init(path, self.map_texture_id, npc_format_fn)
                    .unwrap(),
            );
        }
    }

    pub(crate) fn init(map_texture_id: TextureId) -> Self {
        Self {
            showing: false,
            map_texture_id,
            editor: None,
        }
    }
}
