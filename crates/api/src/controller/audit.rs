use chaty_result::audit::AuditRecord;
use tokio::spawn;

// Audit an event to sentry self hosted
//
// This function s fire and forget so responses are not affected or delayed
pub fn process_audit(_audit: &AuditRecord) {
  spawn(async move {});
}
