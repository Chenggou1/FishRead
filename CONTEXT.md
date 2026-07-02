# FishRead

FishRead is a local EPUB reading runtime. This glossary names the domain boundaries that keep the runtime, CLI protocol, and extensions aligned.

## Language

**Extension**:
A FishRead client that reads from the local reading runtime through the CLI JSON Protocol.
_Avoid_: Plugin, frontend, consumer

**UI Package**:
A FishRead interface package that presents reading workflows to a host environment and depends on the FishRead SDK for runtime data.
_Avoid_: Frontend, client

**FishRead UI Surface**:
A visible interface element that FishRead actively contributes to a host environment through a UI Package.
_Avoid_: Custom UI, frontend element, widget

**Boss Key Hidden State**:
A FishRead privacy state in which FishRead UI Surfaces are hidden and FishRead interactions are suspended, except for the restore action.
_Avoid_: Invisible mode, disabled mode, minimized UI

**FishRead SDK**:
The shared integration surface used by UI Packages to consume FishRead runtime data through the CLI JSON Protocol.
_Avoid_: UI, extension, runtime

**CLI JSON Protocol**:
The stable JSON contract exposed by the FishRead CLI for extensions to consume reading runtime data and errors.
_Avoid_: CLI output, API response shape

**Protocol Version**:
The compatibility version of the CLI JSON Protocol, independent from CLI package, npm package, and Rust crate release versions.
_Avoid_: Package version, crate version, app version

**Reading Position**:
A location in a book where reading can resume or navigation can land.
_Avoid_: Cursor, page, offset

**Current Book**:
The book selected as the active reading target for FishRead commands and UI Packages.
_Avoid_: Open book, active file, selected EPUB

**Reading Anchor**:
A user-facing navigation target inside a chapter that represents a recognizable reading location, maps to a concrete Reading Position, and can be shown with nearby preview text. Reading Anchor labels use chapter-relative percentages.
_Avoid_: Page, chunk, percent point, table-of-contents item

**Reading Navigation**:
A user-facing table of contents that keeps the book's chapter structure and adds Reading Anchors under each chapter for finer navigation.
_Avoid_: Anchor list, chunk list, alternative table of contents
