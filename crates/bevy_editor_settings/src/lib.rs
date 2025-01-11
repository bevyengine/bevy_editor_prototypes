//! A straightforward way to store and retrieve user preferences on disk for Bevy applications.

use bevy::prelude::*;

mod file_system;

/// Annotation for a type to show which type of settings it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum SettingsType {
    /// These are settings that are saved in the os user's configuration directory. \
    /// These settings are global to the user and are not tied to a specific project. \
    /// Settings are along the lines of hotkeys etc.
    Global,
    /// Workspace preferences use the global preferences by default. End users can modify them, customizing their layout, theming and hotkeys. \
    /// The file is created when the user applies changes to their workspace preferences within the editor. \
    /// Workspace preferences can be shared between multiple projects and are not isolated to project folders.*
    Workspace,
    /// Project preference overrides are empty and stored within the project settings. \
    ///  When a project overrides a global/workspace preference, it is no longer possible to change them. \
    ///  In order to modify the preference, users must modify the project settings instead.
    /// There are two states that overrides can be in:
    /// - Inheriting - No override is set. Users can freely change the preference. Users can use what they have set within the global/workspace preferences.
    /// - Modified - When an override has been set, users can no longer change the preference without modifying the project settings. You can switch between inheriting and modified at any time without consequence.
    Project,
}

#[derive(Debug, Clone, Reflect, Default)]
/// Annotation for a type to show how to merge lists when loading settings.
/// if not set, the default is to replace the existing list.
pub enum MergeStrategy {
    #[default]
    /// When Mergeing the list, the new list will replace the existing list.
    Replace,
    /// When Mergeing the list, the new list will be appended to the existing list.
    Append,
}

#[derive(Debug, Clone, Reflect)]
/// Annotation for a type to add tags to the settings. these tags can be used to filter settings in the editor.
pub struct SettingsTags(pub Vec<&'static str>);

#[derive(Debug, Clone, Reflect)]
/// Annotation for a type to add what key the setting should be stored under. if not set the snake case of the type name will be used.
pub struct SettingKey(pub &'static str);

#[derive(Resource)]
/// Store the path for the global preferences directory.
pub struct GlobalSettingsPath(pub std::path::PathBuf);

/// A Bevy plugin for editor settings.
/// This plugin loads the workspace settings, user settings, and project settings.
pub struct EditorSettingsPlugin;

impl Plugin for EditorSettingsPlugin {
    fn build(&self, app: &mut App) {
        match file_system::global_settings_path() {
            Some(path) => {
                debug!("Global settings path: {:?}", path);
                app.insert_resource(GlobalSettingsPath(path));
            }
            None => {
                warn!("Failed to load global settings");
            }
        };
    }

    fn finish(&self, app: &mut App) {
        file_system::load_settings(app);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tracing_test::traced_test;

    #[derive(Debug, Clone, PartialEq, Eq, Reflect, Resource)]
    #[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
    struct BasicSettings {
        pub name: String,
        pub age: u32,
    }

    #[traced_test]
    #[test]
    fn basic_test() {
        let mut app = App::new();

        app.register_type::<BasicSettings>();

        app.insert_resource(BasicSettings {
            name: "John".to_string(),
            age: 25,
        });

        file_system::load_project_settings(app.world_mut());

        let settings = app.world().get_resource::<BasicSettings>().unwrap();

        assert_eq!(settings.name, "bevy_editor_settings");
        assert_eq!(settings.age, 25);
    }

    #[derive(Debug, Clone, PartialEq, Eq, Reflect, Resource)]
    #[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
    struct ListTesting {
        pub list: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Reflect, Resource)]
    #[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
    struct ListTestingAppend {
        #[reflect(@MergeStrategy::Append)]
        pub list: Vec<i32>,
    }

    #[traced_test]
    #[test]
    fn test_lists() {
        let mut app = App::new();

        app.register_type::<ListTesting>();
        app.register_type::<ListTestingAppend>();

        app.insert_resource(ListTesting {
            list: vec!["one".to_string(), "two".to_string()],
        });

        app.insert_resource(ListTestingAppend { list: vec![1, 2] });

        file_system::load_project_settings(app.world_mut());

        let settings = app.world().get_resource::<ListTesting>().unwrap();

        assert_eq!(settings.list, vec!["three".to_string(), "four".to_string()]);

        let settings = app.world().get_resource::<ListTestingAppend>().unwrap();

        assert_eq!(settings.list, vec![1, 2, 3, 4]);
    }

    #[derive(Debug, Clone, PartialEq, Eq, Reflect, Resource)]
    #[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
    enum EnumTesting {
        One,
        Two,
        Three,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Reflect)]
    #[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
    enum EnumTestingField {
        Unit,
        Tuple(String, i32),
        Struct { name: String, age: i32 },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Reflect, Resource)]
    #[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
    struct EnumSettings {
        pub test1: EnumTestingField,
        pub test2: EnumTestingField,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Reflect, Resource)]
    #[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
    struct EnumSettingsList {
        #[reflect(@MergeStrategy::Append)]
        pub settings: Vec<EnumTesting>,
    }

    #[traced_test]
    #[test]
    fn test_enum() {
        let mut app = App::new();

        app.register_type::<EnumTesting>();
        app.register_type::<EnumSettings>();
        app.register_type::<EnumSettingsList>();

        app.insert_resource(EnumTesting::One);
        app.insert_resource(EnumSettings {
            test1: EnumTestingField::Unit,
            test2: EnumTestingField::Unit,
        });
        app.insert_resource(EnumSettingsList {
            settings: vec![EnumTesting::One, EnumTesting::Two],
        });

        file_system::load_project_settings(app.world_mut());

        let settings = app.world().get_resource::<EnumTesting>().unwrap();

        assert_eq!(*settings, EnumTesting::Two);

        let settings = app.world().get_resource::<EnumSettings>().unwrap();

        assert_eq!(
            *settings,
            EnumSettings {
                test1: EnumTestingField::Tuple("hello".to_string(), 42),
                test2: EnumTestingField::Struct {
                    name: "four".to_string(),
                    age: 4,
                },
            }
        );

        let settings = app.world().get_resource::<EnumSettingsList>().unwrap();

        assert_eq!(
            settings.settings,
            vec![EnumTesting::One, EnumTesting::Two, EnumTesting::Three]
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq, Reflect, Resource)]
    #[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
    struct TupleStruct(i32, String);

    #[derive(Debug, Clone, PartialEq, Eq, Reflect, Resource)]
    #[reflect(@SettingsType::Project, @SettingsTags(vec!["basic", "settings", "testing"]))]
    struct StructWithTuple {
        pub tuple: TupleStruct,
    }

    #[traced_test]
    #[test]
    fn test_tuple_struct() {
        let mut app = App::new();

        app.register_type::<TupleStruct>();
        app.register_type::<StructWithTuple>();

        app.insert_resource(TupleStruct(1, "one".to_string()));
        app.insert_resource(StructWithTuple {
            tuple: TupleStruct(2, "two".to_string()),
        });

        file_system::load_project_settings(app.world_mut());

        let settings = app.world().get_resource::<TupleStruct>().unwrap();

        assert_eq!(*settings, TupleStruct(2, "two".to_string()));

        let settings = app.world().get_resource::<StructWithTuple>().unwrap();

        assert_eq!(
            *settings,
            StructWithTuple {
                tuple: TupleStruct(3, "three".to_string()),
            }
        );
    }
}
