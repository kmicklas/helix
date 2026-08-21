use helix_event::{register_hook, send_blocking};
use helix_view::events::DiagnosticsDidChange;
use helix_view::handlers::diagnostics::DiagnosticEvent;
use helix_view::handlers::Handlers;

pub(super) fn register_hooks(_handlers: &Handlers) {
    register_hook!(move |event: &mut DiagnosticsDidChange<'_>| {
        for (view, _) in event.editor.tree.views_mut() {
            send_blocking(&view.diagnostics_handler.events, DiagnosticEvent::Refresh)
        }
        Ok(())
    });
}
