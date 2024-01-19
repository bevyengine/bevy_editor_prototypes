use crate::*;
use bevy::prelude::*;

/// All systems for editor ui wil be placed in UiSystemSet
#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct UiSystemSet;

/// Plugin for editor ui
pub struct EditorUiPlugin {
    pub use_standard_layout: bool,
}

impl Default for EditorUiPlugin {
    fn default() -> Self {
        Self {
            use_standard_layout: true,
        }
    }
}

/// State to determine if editor ui should be shown (ot hidden for any reason)
#[derive(Hash, PartialEq, Eq, Debug, Clone, States, Default)]
pub enum ShowEditorUi {
    #[default]
    Show,
    Hide,
}

impl FlatPluginList for EditorUiPlugin {
    fn add_plugins_to_group(&self, group: PluginGroupBuilder) -> PluginGroupBuilder {
        let mut res = group
            .add(SelectedPlugin)
            .add(EditorUiCore::default())
            .add(GameViewPlugin)
            .add(bottom_menu::BottomMenuPlugin)
            .add(MouseCheck)
            .add(CameraViewTabPlugin)
            .add(SpaceHierarchyPlugin::default())
            .add(SpaceInspectorPlugin)
            .add(GizmoToolPlugin)
            .add(ChangeChainViewPlugin)
            .add(settings::SettingsWindowPlugin);

        if self.use_standard_layout {
            res = res.add(DefaultEditorLayoutPlugin);
        }

        res
    }
}

impl PluginGroup for EditorUiPlugin {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = self.add_plugins_to_group(group);
        group
    }
}

pub struct DefaultEditorLayoutPlugin;

impl Plugin for DefaultEditorLayoutPlugin {
    fn build(&self, app: &mut App) {
        let mut editor = app.world.resource_mut::<EditorUi>();
        editor.tree = egui_dock::DockState::new(vec![EditorTabName::GameView]);

        let [_game, _inspector] = editor.tree.main_surface_mut().split_right(
            egui_dock::NodeIndex::root(),
            0.8,
            vec![EditorTabName::Inspector],
        );
        let [_hierarchy, _game] =
            editor
                .tree
                .main_surface_mut()
                .split_left(_game, 0.2, vec![EditorTabName::Hierarchy]);
    }
}

pub struct EditorUiCore {
    pub disable_no_editor_cams: bool,
}

impl Default for EditorUiCore {
    fn default() -> Self {
        Self {
            disable_no_editor_cams: true,
        }
    }
}

impl Plugin for EditorUiCore {
    fn build(&self, app: &mut App) {
        app.add_state::<ShowEditorUi>();

        app.configure_sets(
            Update,
            UiSystemSet
                .in_set(EditorSet::Editor)
                .run_if(in_state(EditorState::Editor).and_then(in_state(ShowEditorUi::Show))),
        );
        app.init_resource::<EditorUi>();
        app.init_resource::<ScheduleEditorTabStorage>();
        app.add_systems(
            Update,
            (
                show_editor_ui
                    .before(update_pan_orbit)
                    .before(ui_camera_block)
                    .after(bottom_menu::menu),
                set_camera_viewport,
            )
                .in_set(UiSystemSet),
        );

        app.add_systems(
            PostUpdate,
            set_camera_viewport
                .run_if(has_window_changed)
                .in_set(UiSystemSet),
        );
        app.add_systems(
            Update,
            reset_camera_viewport.run_if(in_state(EditorState::Game)),
        );
        app.add_systems(OnEnter(ShowEditorUi::Hide), reset_camera_viewport);
        app.editor_tab_by_trait(EditorTabName::GameView, GameViewTab::default());

        app.editor_tab_by_trait(
            EditorTabName::Other("Debug World Inspector".to_string()),
            self::debug_panels::DebugWorldInspector {},
        );

        app.init_resource::<EditorLoader>();

        app.insert_resource(EditorCameraEnabled(true));

        app.add_systems(
            Startup,
            (set_start_state, apply_state_transition::<EditorState>).chain(),
        );

        //play systems
        app.add_systems(OnEnter(EditorState::GamePrepare), save_prefab_before_play);
        app.add_systems(
            OnEnter(SaveState::Idle),
            to_game_after_save.run_if(in_state(EditorState::GamePrepare)),
        );

        app.add_systems(OnEnter(EditorState::Game), change_camera_in_play);

        app.add_systems(
            OnEnter(EditorState::Editor),
            (clear_and_load_on_start, set_camera_viewport),
        );

        app.add_systems(
            Update,
            (draw_camera_gizmo, selection::delete_selected)
                .run_if(in_state(EditorState::Editor).and_then(in_state(ShowEditorUi::Show))),
        );

        if self.disable_no_editor_cams {
            app.add_systems(
                Update,
                disable_no_editor_cams.run_if(in_state(EditorState::Editor)),
            );

            app.add_systems(OnEnter(EditorState::Editor), change_camera_in_editor);
        }

        app.add_event::<selection::SelectEvent>();

        app.init_resource::<BundleReg>();
    }
}

/// This system use to show all egui editor ui on primary window
/// Will be usefull in some specific cases to ad new system before/after this system
pub fn show_editor_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<EditorUi, _>(|world, mut editor_ui| {
        editor_ui.ui(world, egui_context.get_mut());
    });
}

/// This resource contains registered editor tabs and current dock tree state
#[derive(Resource)]
pub struct EditorUi {
    pub registry: HashMap<EditorTabName, EditorUiReg>,
    pub tree: egui_dock::DockState<EditorTabName>,
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            registry: HashMap::default(),
            tree: egui_dock::DockState::new(vec![]),
        }
    }
}

/// This enum determine how tab was registered.
/// ResourceBased - tab will be registered as resource
/// Schedule - tab will be registered as system
pub enum EditorUiReg {
    ResourceBased {
        show_command: EditorTabShowFn,
        title_command: EditorTabGetTitleFn,
    },
    Schedule,
}

impl EditorUi {
    pub fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        //collect tab names to vec to detect visible
        let mut visible = vec![];
        for (_surface_index, tab) in self.tree.iter_all_nodes() {
            match tab {
                egui_dock::Node::Empty => {}
                egui_dock::Node::Leaf {
                    rect: _,
                    viewport: _,
                    tabs,
                    active: _,
                    scroll: _,
                } => visible.extend(tabs.clone()),
                egui_dock::Node::Vertical {
                    rect: _,
                    fraction: _,
                } => {}
                egui_dock::Node::Horizontal {
                    rect: _,
                    fraction: _,
                } => {}
            }
        }

        let cell = world.as_unsafe_world_cell();

        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, unsafe { cell.world() });

        let mut tab_viewer = unsafe {
            EditorTabViewer {
                commands: &mut commands,
                world: cell.world_mut(),
                registry: &mut self.registry,
                visible,
                tab_commands: vec![],
            }
        };

        DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show(ctx, &mut tab_viewer);

        let windows_setting = unsafe { cell.world_mut().resource_mut::<NewWindowSettings>() };
        for command in tab_viewer.tab_commands {
            match command {
                EditorTabCommand::Add {
                    name,
                    surface,
                    node,
                } => match windows_setting.new_tab {
                    NewTabBehaviour::Pop => {
                        self.tree.add_window(vec![name]);
                    }
                    NewTabBehaviour::SameNode => {
                        if let Some(tree) = self
                            .tree
                            .get_surface_mut(surface)
                            .and_then(|surface| surface.node_tree_mut())
                        {
                            tree.set_focused_node(node);
                            tree.push_to_focused_leaf(name);
                        }
                    }
                    NewTabBehaviour::SplitNode => {
                        if let Some(surface) = self.tree.get_surface_mut(surface) {
                            surface
                                .node_tree_mut()
                                .unwrap()
                                .split_right(node, 0.5, vec![name]);
                        }
                    }
                },
            }
        }

        unsafe {
            command_queue.apply(cell.world_mut());
        }
    }
}

/// Trait for registering editor tabs via app.**
pub trait EditorUiAppExt {
    fn editor_tab_by_trait<T>(&mut self, tab_id: EditorTabName, tab: T) -> &mut Self
    where
        T: EditorTab + Resource + Send + Sync + 'static;
    fn editor_tab<T>(
        &mut self,
        tab_id: EditorTabName,
        title: egui::WidgetText,
        tab_systesm: impl IntoSystemConfigs<T>,
    ) -> &mut Self;
}

impl EditorUiAppExt for App {
    fn editor_tab_by_trait<T>(&mut self, tab_id: EditorTabName, tab: T) -> &mut Self
    where
        T: EditorTab + Resource + Send + Sync + 'static,
    {
        self.insert_resource(tab);
        let show_fn = Box::new(
            |ui: &mut egui::Ui, commands: &mut Commands, world: &mut World| {
                world.resource_scope(|scoped_world, mut data: Mut<T>| {
                    data.ui(ui, commands, scoped_world)
                });
            },
        );
        let reg = EditorUiReg::ResourceBased {
            show_command: show_fn,
            title_command: Box::new(|world| world.resource_mut::<T>().title()),
        };

        self.world
            .resource_mut::<EditorUi>()
            .registry
            .insert(tab_id, reg);
        self
    }

    fn editor_tab<T>(
        &mut self,
        tab_id: EditorTabName,
        title: egui::WidgetText,
        tab_systesm: impl IntoSystemConfigs<T>,
    ) -> &mut Self {
        let mut tab = ScheduleEditorTab {
            schedule: Schedule::default(),
            title,
        };

        tab.schedule.add_systems(tab_systesm);

        self.world
            .resource_mut::<ScheduleEditorTabStorage>()
            .0
            .insert(tab_id.clone(), tab);
        self.world
            .resource_mut::<EditorUi>()
            .registry
            .insert(tab_id, EditorUiReg::Schedule);
        self
    }
}

/// Temporary resource for pretty system, based tab registration
pub struct EditorUiRef(pub egui::Ui);

/// Sytem to block camera control if egui is using mouse
pub fn ui_camera_block(
    mut ctxs: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut state: ResMut<EditorCameraEnabled>,
    game_view: Res<GameViewTab>,
) {
    let Ok(mut ctx_ref) = ctxs.get_single_mut() else {
        return;
    };
    let ctx = ctx_ref.get_mut();
    if ctx.is_using_pointer() || ctx.is_pointer_over_area() {
        let Some(pos) = ctx.pointer_latest_pos() else {
            return;
        };
        if let Some(area) = game_view.viewport_rect {
            if area.contains(pos) {
            } else {
                *state = EditorCameraEnabled(false);
            }
        } else {
            *state = EditorCameraEnabled(false);
        }
    }
}
