// Remote driver -- most remote tests test classify_git_error() which is a pure function
// not requiring TestContext. The driver is minimal; classify_git_error is called directly
// in tests since it has no state dependency.
use crate::common::context::TestContext;

// Placeholder to satisfy the module declaration. TestContext methods for async remote
// operations (fetch, pull, push) are deferred to Phase 55 (Integration Testing).
// The classify_git_error function is a pure function tested via direct calls.
#[allow(unused_imports)]
use trunk_lib::commands::remote;
