# Design Paradigms

## Non-Overlapping

The Ui should enable you to always view all relevant information and tools at any given moment, you should never have to move a window out of your way, for that reason, as well as sticking with bevy's modular structure, Bevy uses a divided window layout we call Panes.

1. Screens; the entire window enables you to configure workspaces using multiple editors.
2. Panes; the container of an editor. Editors can each operate similar to a stand alone editor, like for modeling, painting or scripting. Editors can both follow Screen level user context ("Active" or "Selected") and offer local context browsing.
3. Regions; every editor allows further subdivision to provide button/menu headers, toolbars, channel views, and what more is needed to serve its functionality.

## Non-Blocking

Tools and interface options are being designed to not block the user from using any other parts of Bevy Editor. Bevy Editor doesn't pop-up forms that the user must fill. The UI should stay responsive by all means, at least for the common and most used operations. When things exceptionally do block (Like Running the game) it should be clearly indicated and allow an immediate exit. User's don't like their workflow being interrupted.

## Non-Modal

User input should remain as consistent and predictable as possible, do not change what inputs do on the fly without clear communication.

All input behavior changes should be "Temporary" modes, immediately ending the operation when a user stops with actions.

## Select -> Operate

In Blender you first indicate which data you work on, and then what you want to do. The same applies for Bevy. This follows the non-modal principle; there's no active tool mode you need to set first to be able to use the tool on what you select after. This concept enables a fast and flexible work flow.

## Bevy Editor is for Artists

Bevy Editor is for the people on teams you don't/shouldn't expect to know how to code.
Ex: A level designer should not have to touch Rust just to place a door.
