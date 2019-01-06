use super::event::*;
use super::action::*;
use super::view_state::*;

use flo_ui::*;
use flo_ui::session::*;

use std::sync::*;
use std::collections::HashMap;

///
/// Represents the type
///
pub struct AppState {
    /// The root view
    root_view: Option<ViewState>,

    /// The ID that will be assigned to the next view we create
    next_view_id: usize,

    /// The IDs for the viewmodels that we're managing
    view_models: HashMap<Vec<Arc<String>>, usize>,

    /// The IDs for the properties in view models (every name gets a )
    view_model_properties: HashMap<String, usize>,

    /// Maps view IDs to addresses
    address_for_view: HashMap<usize, Vec<Arc<String>>>,

    /// The next viewmodel ID to assign
    next_viewmodel_id: usize,

    /// The next ID to assign to a property
    next_property_id: usize
}

impl AppState {
    ///
    /// Creates a new AppState
    ///
    pub fn new() -> AppState {
        AppState {
            root_view:              None,
            view_models:            HashMap::new(),
            view_model_properties:  HashMap::new(),
            address_for_view:       HashMap::new(),
            next_view_id:           0,
            next_viewmodel_id:      0,
            next_property_id:       0
        }
    }

    ///
    /// Changes a UI update into one or more AppActions
    ///
    pub fn map_update(&mut self, update: UiUpdate) -> Vec<AppAction> {
        use self::UiUpdate::*;

        match update {
            Start                       => { self.start() }
            UpdateUi(differences)       => { self.update_ui(differences) }
            UpdateCanvas(differences)   => { vec![] }
            UpdateViewModel(updates)    => { self.update_viewmodel(updates) }
        }
    }

    ///
    /// Changes an AppEvent into a UiEvent
    ///
    pub fn map_event(&mut self, update: AppEvent) -> Vec<UiEvent> {
        use self::AppEvent::*;

        match update {
            Click(view_id, name)    => vec![UiEvent::Action(self.get_controller_path_for_view(view_id), name, ActionParameter::None)]
        }
    }

    ///
    /// Processes the 'start' update
    ///
    fn start(&mut self) -> Vec<AppAction> {
        vec![
            AppAction::CreateWindow(0),
            AppAction::Window(0, WindowAction::Open)
        ]
    }

    ///
    /// Retrieves the controller path for a particular view ID
    ///
    fn get_controller_path_for_view(&self, view_id: usize) -> Vec<String> {
        self.address_for_view.get(&view_id)
            .map(|address| address.iter().map(|component| (**component).clone()).collect())
            .unwrap_or_else(|| vec![])
    }

    ///
    /// Maps a UiDiff into the AppActions required to carry it out
    ///
    fn update_ui(&mut self, differences: Vec<UiDiff>) -> Vec<AppAction> {
        differences.into_iter()
            .flat_map(|diff| self.update_ui_from_diff(diff))
            .collect()
    }

    ///
    /// Removes the settings for a view from this state
    ///
    fn remove_view(view_state: &ViewState, address_for_view: &mut HashMap<usize, Vec<Arc<String>>>) {
        // Remove all of the subviews first
        for subview in view_state.subviews() {
            Self::remove_view(subview, address_for_view);
        }

        // Remove the settings for this view
        address_for_view.remove(&view_state.id());
    }

    ///
    /// Returns the actions required to perform a single UI diff
    ///
    fn update_ui_from_diff(&mut self, difference: UiDiff) -> Vec<AppAction> {
        // Get the controller path
        let controller_path = self.root_view.as_ref().map(|root_view| root_view.get_controller_path_at_address(&difference.address)).unwrap_or(vec![]);

        // Create the replacement view states
        let (view_state, mut actions) = self.create_view(&difference.new_ui, &controller_path);

        // The difference specifies a view to replace
        let root_view           = &mut self.root_view;
        let address_for_view    = &mut self.address_for_view;
        let view_to_replace     = root_view.as_ref().and_then(|root_view| root_view.get_state_at_address(&difference.address));

        // Generate the actions to remove the existing view
        actions.extend(view_to_replace.map(|view_to_replace| view_to_replace.destroy_subtree_actions()).unwrap_or(vec![]));

        // Remove the data for the view
        view_to_replace.map(|view_to_replace| Self::remove_view(view_to_replace, address_for_view));

        // Replace with the new state
        if difference.address.len() > 0 {
            // Add as a subview of the view
            let mut parent_address  = difference.address.clone();
            parent_address.pop();
            let parent_view         = self.root_view.as_ref().and_then(|root_view| root_view.get_state_at_address(&parent_address));

            parent_view.map(|parent_view| actions.push(AppAction::View(parent_view.id(), ViewAction::AddSubView(view_state.id()))));

            self.root_view.as_mut().map(|root_view| root_view.replace_child_state(&difference.address, view_state));
        } else {
            // Add as the root view
            actions.push(AppAction::Window(0, WindowAction::SetRootView(view_state.id())));
            self.root_view = Some(view_state);
        }

        actions
    }

    ///
    /// Converts a UI property into an AppProperty binding
    /// 
    /// This returns the property binding and any AppActions that might be required to ensure that it's valid.
    /// This means that if there is no viewmodel for the specified controller path and the property requires one,
    /// the actions will be amended to create one.
    ///
    fn app_property(&mut self, controller_path: &Vec<Arc<String>>, property: Property) -> (Vec<AppAction>, AppProperty) {
        use self::Property::*;

        match property {
            Nothing     => (vec![], AppProperty::Nothing),
            Bool(val)   => (vec![], AppProperty::Bool(val)),
            Int(val)    => (vec![], AppProperty::Int(val)),
            Float(val)  => (vec![], AppProperty::Float(val)),
            String(val) => (vec![], AppProperty::String(val)),

            Bind(name)  => {
                // Fetch or create the viewmodel ID
                let mut actions     = vec![];
                let viewmodel_id    = if let Some(viewmodel_id) = self.view_models.get(controller_path) {
                    // Use the existing ID
                    *viewmodel_id
                } else {
                    // Create a new ID
                    let viewmodel_id = self.next_viewmodel_id;
                    self.next_viewmodel_id += 1;
                    self.view_models.insert(controller_path.clone(), viewmodel_id);

                    // Send actions to create the viewmodel
                    actions.push(AppAction::CreateViewModel(viewmodel_id));

                    viewmodel_id
                };

                // Fetch or create the property ID
                let property_id     = self.create_or_retrieve_property_id(&name);

                // Generate the resulting app property
                (actions, AppProperty::Bind(viewmodel_id, property_id))
            }
        }
    }

    ///
    /// Creates a view (and subviews) from a UI control
    ///
    fn create_view(&mut self, control: &Control, controller_path: &Vec<Arc<String>>) -> (ViewState, Vec<AppAction>) {
        // Create a new view state
        let view_id                 = self.next_view_id;
        self.next_view_id           += 1;
        let mut view_state          = ViewState::new(view_id);

        // Store the controller path for this view
        self.address_for_view.insert(view_id, controller_path.clone());

        // Initialise from the control
        let mut property_actions    = vec![];
        let setup_actions           = view_state.set_up_from_control(control, |property| {
            let (actions, property) = self.app_property(controller_path, property);

            property_actions.extend(actions);

            property
        });

        // Property setup actions need to occur before all the other actions associated with this control's setup
        property_actions.extend(setup_actions);
        let mut setup_actions = property_actions;

        // Work out the controller path for the subcomponents. If the view state has a controller, then add it to the existing path, otherwise keep the existing path
        let mut edited_controller_path;

        let subcomponent_controller_path    = if let Some(subview_controller) = view_state.get_subview_controller() {
            edited_controller_path = controller_path.clone();
            edited_controller_path.push(subview_controller);
            &edited_controller_path
        } else {
            controller_path
        };

        // Also set up any subcomponents
        for subcomponent in control.subcomponents().unwrap_or(&vec![]) {
            // Create the view for the subcomponent
            let (subcomponent_view, subcomponent_actions) = self.create_view(subcomponent, subcomponent_controller_path);

            // Add to the setup actions
            setup_actions.extend(subcomponent_actions);

            // Add as a subview
            setup_actions.push(AppAction::View(view_id, ViewAction::AddSubView(subcomponent_view.id())));

            // Add as a child control of our view state
            view_state.add_child_state(subcomponent_view);
        }

        (view_state, setup_actions)
    }

    ///
    /// Retrieves or creates the property ID for the specified name
    ///
    fn create_or_retrieve_property_id(&mut self, property_name: &str) -> usize {
        if let Some(id) = self.view_model_properties.get(property_name) {
            *id
        } else {
            // Assigned a new ID
            let id = self.next_property_id;
            self.next_property_id += 1;
            self.view_model_properties.insert(String::from(property_name), id);

            id
        }
    }

    ///
    /// Performs a viewmodel update
    ///
    fn update_viewmodel(&mut self, updates: Vec<ViewModelUpdate>) -> Vec<AppAction> {
        updates.into_iter()
            .flat_map(|update| self.perform_viewmodel_update(update))
            .collect()
    }

    ///
    /// Performs a single viewmodel update
    ///
    fn perform_viewmodel_update(&mut self, update: ViewModelUpdate) -> Vec<AppAction> {
        let mut actions = vec![];

        // Retrieve the viewmodel for this controller
        let controller_path = update.controller_path()
            .iter()
            .map(|path_item| Arc::new(path_item.clone()))
            .collect();

        let viewmodel_id = if let Some(viewmodel_id) = self.view_models.get(&controller_path) {
            // Use the existing ID
            *viewmodel_id
        } else {
            // Create a new ID
            let viewmodel_id = self.next_viewmodel_id;
            self.next_viewmodel_id += 1;
            self.view_models.insert(controller_path, viewmodel_id);

            // Send actions to create the viewmodel
            actions.push(AppAction::CreateViewModel(viewmodel_id));

            viewmodel_id
        };

        // Add the changes to the actions
        for change in update.updates().iter() {
            use self::ViewModelChange::*;

            match change {
                NewProperty(name, value)      => {
                    let property_id = self.create_or_retrieve_property_id(name);
                    actions.push(AppAction::ViewModel(viewmodel_id, ViewModelAction::CreateProperty(property_id)));
                    actions.push(AppAction::ViewModel(viewmodel_id, ViewModelAction::SetPropertyValue(property_id, value.clone())));

                },

                PropertyChanged(name, value)  => {
                    let property_id = self.create_or_retrieve_property_id(name);
                    actions.push(AppAction::ViewModel(viewmodel_id, ViewModelAction::SetPropertyValue(property_id, value.clone())));
                }
            }
        }

        actions
    }
}
