use super::state::*;
use super::event::*;
use super::super::control::*;
use super::super::controller::*;

use binding::*;

use std::mem;
use std::sync::*;

///
/// Core UI session structures
/// 
pub struct UiSessionCore {
    /// The sequential ID of the last wake for update event
    last_update_id: u64,

    /// The UI tree for the applicaiton
    ui_tree: BindRef<Control>,

    /// Functions to be called next time the core is updated
    update_callbacks: Vec<Box<Fn(&mut UiSessionCore) -> ()+Send>>
}

impl UiSessionCore {
    ///
    /// Creates a new UI core
    /// 
    pub fn new(controller: Arc<Controller>) -> UiSessionCore {
        // Assemble the UI for the controller
        let ui_tree = assemble_ui(controller);

        UiSessionCore {
            last_update_id:     0,
            ui_tree:            ui_tree,
            update_callbacks:   vec![]
        }
    }

    ///
    /// Retrieves the ID of the last update that was dispatched for this core
    /// 
    pub fn last_update_id(&self) -> u64 { self.last_update_id }

    ///
    /// Retrieves a reference to the UI tree for the whole application
    /// 
    pub fn ui_tree(&self) -> BindRef<Control> { BindRef::clone(&self.ui_tree) }

    ///
    /// Dispatches an event to the specified controller
    ///  
    pub fn dispatch_event(&mut self, event: UiEvent, controller: &Controller) {
        // Send the event to the controllers
        match event {
            UiEvent::Action(controller_path, event_name, action_parameter) => {
                // Find the controller along this path
                if controller_path.len() == 0 {
                    // Straight to the root controller
                    self.dispatch_action(controller, event_name, action_parameter);
                } else {
                    // Controller along a path
                    let mut controller = controller.get_subcontroller(&controller_path[0]);

                    for controller_name in controller_path.into_iter().skip(1) {
                        controller = controller.map_or(None, move |ctrl| ctrl.get_subcontroller(&controller_name));
                    }

                    match controller {
                        Some(ref controller)    => self.dispatch_action(&**controller, event_name, action_parameter),
                        None                    => ()       // TODO: event has disappeared into the void :-(
                    }
                }
            },

            UiEvent::Tick => {
                // Send a tick to this controller
                self.dispatch_tick(controller);
            }
        }

        // It might be time to wake anything waiting on the update stream
        self.wake_for_updates();
    }

    ///
    /// Registers a function to be called next time the core is updated
    /// 
    pub fn on_next_update<Callback: 'static+Fn(&mut UiSessionCore) -> ()+Send>(&mut self, callback: Callback) {
        // Call the function when the next update occurs
        self.update_callbacks.push(Box::new(callback))
    }

    ///
    /// Wakes things up that might be waiting for updates
    /// 
    fn wake_for_updates(&mut self) {
        // Update the last update ID
        self.last_update_id += 1;

        // Perform the callbacks
        let mut callbacks = vec![];
        mem::swap(&mut callbacks, &mut self.update_callbacks);

        for callback in callbacks {
            (*callback)(self);
        }
    }

    ///
    /// Dispatches an action to a controller
    /// 
    fn dispatch_action(&mut self, controller: &Controller, event_name: String, action_parameter: ActionParameter) {
        controller.action(&event_name, &action_parameter);
    }

    ///
    /// Sends ticks to the specified controller and all its subcontrollers
    /// 
    fn dispatch_tick(&mut self, controller: &Controller) {
        // Send ticks to the subcontrollers first
        let ui              = controller.ui().get();
        let subcontrollers  = ui.all_controllers();
        for subcontroller_name in subcontrollers {
            if let Some(subcontroller) = controller.get_subcontroller(&subcontroller_name) {
                self.dispatch_tick(&*subcontroller);
            }
        }

        // Send the tick to the controller
        controller.tick();
    }
}
