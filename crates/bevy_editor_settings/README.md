# Bevy Editor Settings

This crate provides a way to have settings for the editor.
There are three types of settings in order of precedence:
- User settings
- Workspace settings
- Default settings

## User settings
User settings are settings that are specific to the user. They are stored in the project at the top level in `user.toml`
these will be moved when the editor is a standalone application

## Workspace settings
Workspace settings are settings that are specific to the workspace. They are stored in the project at the top level in `Bevy.toml`

## Default settings
Default settings are the settings chosen by the plugins or the editor and are stored in the code.


## TODO

- [x] Add a settings editor
- [ ] Add a way for plugins(part of the editor) to add and fetch settings ([bevy_basic_prefs](https://github.com/viridia/bevy_basic_prefs) should be used as a point of start)