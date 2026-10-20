//! Unslid `RobloxLib` addresses for 2.743.1134, documented in `re/notebook`.

pub const PRINT: usize = 0x4356D9C;
pub const WHSJ_VTABLE: usize = 0x60CE758;
pub const WHSJ_STEP: usize = 0x19D4128;
pub const STEP_SLOT: usize = 9;
pub const GET_GLOBAL_STATE: usize = 0x197CD14;
pub const SANDBOX_THREAD: usize = 0x19C0DBC;
pub const SET_PROTO_CAPABILITIES: usize = 0x1978DE0;
pub const TASK_DEFER: usize = 0x195AE44;
pub const LUA_NEWTHREAD: usize = 0x4306BA0;
pub const LUA_SETTOP: usize = 0x4306CC0;
pub const LUA_TOLSTRING: usize = 0x43078E8;
pub const LUA_REPLACE: usize = 0x4306F4C;
pub const LUAU_LOAD: usize = 0x43355E0;

/// `BL sigguard` in the `luau_load` body, replaced by `MOV W0, #0`.
pub const SIGGUARD_CALL: usize = 0x4335850;
pub const SIGGUARD_CALL_WORD: u32 = 0x975841BB;
pub const SIGGUARD_PASS_WORD: u32 = 0x52800000;

pub const JOB_DATA_MODEL: usize = 0x188;
pub const JOB_SCRIPT_CONTEXT: usize = 0x1B0;
pub const DATA_MODEL_PLACE_ID: usize = 0x368;
pub const DATA_MODEL_LOADED: usize = 0x5D8;
pub const THREAD_DATA: usize = 0x8;
pub const THREAD_IDENTITY: usize = 0x30;
pub const THREAD_CAPABILITIES: usize = 0xA0;
