# FishRead keybindings

FishRead keybindings are part of the Pi UI Package contract. They should be easy to press, avoid common Pi editor shortcuts, and keep privacy-sensitive actions fast enough to use without looking.

## Defaults

| Action | Key | Scope | Notes |
| ---- | ---- | ---- | ---- |
| Boss key | `ctrl+q` | Pi extension | Toggles the Boss Key Hidden State. This is the only FishRead interaction that remains active while hidden. |

## Selection rules

- Prefer one-hand shortcuts for urgent privacy actions.
- Avoid `alt` defaults for letter keys because macOS terminals may emit characters such as `œ` instead of a Pi shortcut.
- Avoid Pi's default editor and application shortcuts. Known conflicts include `ctrl+g` for external editor, `ctrl+f` for cursor right, and `alt+f` for cursor word right.
- Avoid shortcuts that mutate reading state while FishRead is in the Boss Key Hidden State.
- New FishRead shortcuts must describe whether they remain active while hidden. The default answer is no; only the boss key restore action should bypass the hidden-state gate.

## Customization

Pi keybindings can be customized in `~/.pi/agent/keybindings.json`, then reloaded with `/reload` in Pi. If a terminal reserves `ctrl+q`, rebind the FishRead boss key before relying on it.

## Change checklist

When adding or changing a FishRead shortcut:

- Update this document.
- Check the current Pi default keybindings for conflicts.
- Keep the implementation behind the boss key interaction gate unless it is the restore action.
- Run the Pi extension TypeScript check.
