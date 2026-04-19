// Remote driver -- most remote tests test classify_git_error() which is a pure function
// not requiring TestContext. The driver is minimal; classify_git_error is called directly
// in tests since it has no state dependency. TestContext methods for async remote
// operations (fetch, pull, push) are deferred to Phase 55 (Integration Testing).
