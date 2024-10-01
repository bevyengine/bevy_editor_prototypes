# Best Practices

## Keep the UI calm

- Avoid strong changes in colors, big jumps in layouts, or popping elements on simple interactions. For example strong mouse hover highlights can be very flashy and cause distracting visual noise; some subtle changes can be okay.

- Avoid strong accent colors for no reason. Using accent colors when you aren't trying to get the user's attention only creates visual noise and clutter that harms the user experience. The UI should be calm with slight contrasts unless there's good reason to do so. Getting this right early on will do good for long term editor-user communication.

- When ever Ui breaks away from Bevy Editor's design practices, the reasoning should be well thought and documented. Consistency is a double edge sword, it can just as easily harm usability as it helps. Breaking away can be an acceptable trade if there are good reasons to do so, and only then.

## Workflow Oriented

When addressing things in the Ui, focus on actions not things. This means use verbs over nouns. Its common to think everything needs a name and names should be nouns. Think like a user instead, you don't care what a button is, you care what it *does*. What is the quickest, most natural feeling way to describe what the user is trying to do? Verbs are a powerful aspect of language, and often fit mental model better.

- Examples:

  - What is a more natural/usable expression: "Make a vertex selection" or "select a vertex"? "Make a connection between two vertices", "Create an edge between two vertices" or "connect two vertices"?
  - The context menu is an action/workflow oriented UI: A simple right click immediately provides common actions (usually a list of verbs). You don't have to first open a Context (noun) > Common Actions (another noun) menu first.
  - Instead of a Text or Font menu, place text formatting (bold, italic, etc.) in a Format menu.
  - A menu entry like Create Asset Data could be named Package as Asset or Mark as Asset instead.
