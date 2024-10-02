# Design Paradigms

## Bevy is for Teams

Bevy (and especially its Editor) is a tool for cross-disciplinary teams to work together.
Each user will have their own unique strengths and weaknesses and tasks to perform.

That means:

1. Collaboration should be pleasant.
2. The Editor must be customizable to meet the unique needs of each user and project.
3. Art, audio, writing and design-centric workflows should require a minimal level of technical expertise.

## Redundant interaction paradigms for accessibility

1. Anything that can be done via a mouse click can be done via a hotkey.
2. Anything that can be found in a menu can be found in the command palette.
3. Anything that can be configured via setting a menu option can be done by editing a config file.

To be completely clear: the same should be true in the opposite directions!

## Non-Overlapping

The UI should enable you to always view all relevant information and tools at any given moment, you should never have to move a window out of your way, for that reason, as well as sticking with bevy's modular structure, Bevy uses a divided window layout we call Panes.

1. Screens; the entire window enables you to configure workspaces using multiple editors.
2. Panes; the container of an editor. Editors can each operate similar to a stand alone editor, like for modeling, painting or scripting. Editors can both follow Screen level user context ("Active" or "Selected") and offer local context browsing.
3. Regions; every editor allows further subdivision to provide button/menu headers, toolbars, channel views, and what more is needed to serve its functionality.

## Non-Blocking

The UI should stay responsive whenever possible. Tools and interface options should not block the user from using any other parts of Bevy Editor. When long-running blocking tasks are required, it should be clearly indicated and allow an immediate exit. Users don't like their workflow being interrupted!

## Non-Modal

User input should remain as consistent and predictable as possible, do not change what inputs do on the fly without clear communication.

When input behavior changes (e.g. a special camera mode that overrides normal controls), the change in behavior should be temporary, and require continuous user input (typically pressing a key or holding a click) to remain active.

## Select -> Operate

In [Blender](https://developer.blender.org/docs/features/interface/human_interface_guidelines/paradigms/#select-operate) you first indicate which data you work on, and then what you want to do. The same applies for Bevy. Quote from Blender's guidelines: "This follows the non-modal principle; there's no active tool mode you need to set first to be able to use the tool on what you select after. This concept enables a fast and flexible work flow."
