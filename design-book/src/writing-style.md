# Writing Style

## Common Rules

- Bevy uses American English.
- Text should be clear and to the point.
- Prefer plain and simple language and avoid contractions and abbreviations.
- Language should be targeted at the lowest level of technical expertise needed to clearly understand an explanation.
- Entity, component, resource, system, event and asset are the critical technical vocabulary for all Bevy users to learn, no matter what their role is.
- Do not use English contractions like aren’t, can’t, etc.
- Single character identifiers, like X, Y, Z, R, G, B, etc. are always capitalized.
- Do not use implementation specific language.
  - Don't use "int" or "struct"
  - Instead use "Number" or "Data"
- Don't use direct language, such as you. This sounds like an accusation towards the user.
  - Bad: "Your system is slow"
  - Good: "The system is low performance"
- Use the ampersand (&) in labels, but and in text (e.g. in tooltip descriptions).
- Text should generally not contain code snippets, or involved details (e.g. troubleshooting, corner cases that might not work, etc.).

## UI

- Use MLA Title case for UI labels.
- Avoid redundancy. For example do not use Enable, Activate, Is, Use, or similar words for boolean properties, just a checkbox.
