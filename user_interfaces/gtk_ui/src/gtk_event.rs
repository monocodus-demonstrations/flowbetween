use super::gtk_action::*;
use super::gtk_event_parameter::*;

///
/// User interface events that can be generated by Gtk
///
#[derive(Clone, PartialEq, Debug)]
pub enum GtkEvent {
    /// Dummy event (used for testing)
    None,

    /// Finished processing UI events and about to begin waiting again
    Tick,

    /// A window was closed by the user
    CloseWindow(WindowId),

    /// Registered event has occurred on a widget
    Event(WidgetId, String, GtkEventParameter)
}
