//! Unslid `RobloxLib` addresses for 2.743.1134, documented in `re/notebook`.

/// `"Current identity is %d"`, the format passed to it.
pub const PRINT: usize = 0x4356D9C;
/// `"WaitingHybridScriptsJob"`, the vptr stored by its constructor.
pub const WHSJ_VTABLE: usize = 0x60CE758;
/// `stepDataModelJob`, called through slot 9 by `DataModelJob::step`.
pub const WHSJ_STEP: usize = 0x19D4128;
pub const STEP_SLOT: usize = 9;
/// Read by `DataModelJob::step` before it calls the step slot.
pub const JOB_DATA_MODEL: usize = 0x188;
