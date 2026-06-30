# FishRead

FishRead is a local EPUB reading runtime. This glossary names the domain boundaries that keep the runtime, CLI protocol, and extensions aligned.

## Language

**Extension**:
A FishRead client that reads from the local reading runtime through the CLI JSON Protocol.
_Avoid_: Plugin, frontend, consumer

**UI Package**:
A FishRead interface package that presents reading workflows to a host environment and depends on the FishRead SDK for runtime data.
_Avoid_: Frontend, client

**FishRead SDK**:
The shared integration surface used by UI Packages to consume FishRead runtime data through the CLI JSON Protocol.
_Avoid_: UI, extension, runtime

**CLI JSON Protocol**:
The stable JSON contract exposed by the FishRead CLI for extensions to consume reading runtime data and errors.
_Avoid_: CLI output, API response shape

**Protocol Version**:
The compatibility version of the CLI JSON Protocol, independent from CLI package, npm package, and Rust crate release versions.
_Avoid_: Package version, crate version, app version
