# Sunlight Core

The core is the injected dylib. `runtime` repackages the iOS Roblox app as Mac
Catalyst and loads it with `DYLD_INSERT_LIBRARIES`; this is what runs once we
are inside the process.

## Goals

- Lean. unix-roblox is a working executor in ~650 lines; we match that size with
  a cleaner structure.
- Call Roblox's own Luau C API by address. We do **not** relink a struct-matched
  VM (Opiumware's approach), trading a large, brittle struct/pointer-encoding map
  for a short list of function addresses.
- Patch as little as possible, and touch the Lua state only from a thread the
  engine owns.

## First milestone

Print `hiiii from sunlight` on joining a game, in two steps:

1. **Engine reached** (~4 offsets). Find `RobloxLib` and its slide, detect a
   loaded game via the scheduler, call Roblox's internal print.
2. **Script runs** (~15 offsets). Run `print("hiiii from sunlight")` through the
   real path: compile → encode → `luau_load` → identity → `task.defer`.

Step 1 becomes the game-detection and logging of step 2; nothing is thrown away.

## Shape

Two crates: `runtime` (launcher) and `sunlight` (`cdylib`, target
`aarch64-apple-ios-macabi`). Modules in `sunlight`:

- `image` — resolve `RobloxLib` and its ASLR slide by name. unix-roblox hardcodes
  image index 1 and base `0x100000000`, which only holds for the macOS player.
- `offsets` — one struct of addresses resolved at startup, each documented with
  its string anchor so it can be re-found. The auto-dumper fills it later.
- `luau` — thin typed wrappers over the engine's `lua_*` functions. A `State`
  newtype keeps stack discipline.
- `scheduler` — walk the TaskScheduler job list, find `ScriptContext` via
  `WaitingHybridScriptsJob`, derive the global state, report game-loaded.
- `exec` — compile, encode, `luau_load`, set identity with a scoped capability
  mask, `task.defer`. Runs **inside** a scheduler job step, draining a queue,
  never from a foreign thread. unix-roblox executes from a detached background
  thread; this is the main thing to fix.
- `hooks` — the minimum: SigGuard, plus teleport/leave for state reset. A small
  arm64 inline-hook helper instead of vendoring tinyhook.
- `env` — one Rust function per executor global, registered from a single static
  list of `(name, aliases, fn)`.

## Bytecode

`luau_load` calls SigGuard, which rejects our bytecode as "corrupted". We hook
SigGuard but bypass it **only** for our own loads: `exec` sets a thread-local
flag around its `luau_load` call, and the hook returns ok when the flag is set,
otherwise calls the original. SigGuard has a single caller (the loader), found
via one string xref, and runs synchronously on the calling thread, so the bypass
is surgical and Roblox's own loads keep feeding the integrity counters.
unix-roblox and Opiumware replace the check outright, which zeroes those counters
for every load.

Appending a valid trailer so SigGuard passes is a documented fallback only; it
depends on server-controlled flags and the obfuscated trailer format.

Compile-side opcode encoding is plain `op * 227`. Bytecode already in memory uses
Roblox's shuffled opcode numbers, so later bytecode-reading features (`debug.*`,
`getscriptbytecode`) must translate through the engine's opcode table.

In-process `__TEXT` patching is confirmed working under ad-hoc Catalyst signing
via `mach_vm_protect(RW|COPY)` → write → restore `RX` (tinyhook's mechanism).

## Capabilities

unix-roblox grants `0xFFFF...` to every thread. We set identity 8 with a scoped
mask so the vulnerability surface (file URLs, auth headers, blocked service
methods) is handled by policy in the hooks, not by handing scripts everything.
Detail is a later milestone.

## References

- `references/unix-roblox` — the lean baseline. We follow its execution path and
  offset-anchor method, and fix its weak spots: image-index assumption,
  all-capabilities, cross-thread execution.
- `references/Opiumware-Source` — the heavy, struct-matched approach. A map of
  what deeper features need, not a structure to copy.
- `references/roblox-ios-offsets` — stale numbers, but the string-anchor method
  is how we recover offsets per build.
- `references/Myriad` — the conformance target for a **later** step, once the
  core runs scripts. It defines which offsets we map next; it does not shape the
  core.
