//!
//! Routines for converting UI controls to HTML
//!

use ui::*;
use super::minidom::*;

use percent_encoding::*;

///
/// Trait implemented by things that can be represented by HTML
///
pub trait ToHtml {
    ///
    /// Converts this object to HTML. The base path specifies where resources can be found.
    /// 
    fn to_html(&self, base_path: &str) -> DomNode {
        self.to_html_subcomponent(base_path, "")
    }

    ///
    /// Converts this object to HTML, when it's a subcomponent. The controller path is a
    /// string indicating where the controller for this item can be found.
    /// 
    fn to_html_subcomponent(&self, base_path: &str, controller_path: &str) -> DomNode;
}

///
/// Appends a sub-controller to a controller path, returning the new controller path
/// 
pub fn append_component_to_controller_path<'a>(controller_path: &str, subcontroller_name: &str) -> String {
    format!("{}/{}", controller_path, utf8_percent_encode(subcontroller_name, DEFAULT_ENCODE_SET))
}

///
/// Given a UI tree and an address, returns the controller path for that component
/// 
pub fn html_controller_path_for_address<'a>(ui_tree: &'a Control, address: &Vec<u32>) -> String {
    let controller_path = controller_path_for_address(ui_tree, address);

    controller_path.iter().fold(String::new(), |path, &component| append_component_to_controller_path(&path, component))
}

///
/// Returns the class for a control
///
fn control_class(ctrl: &Control) -> &str {
    use ui::ControlType::*;

    match ctrl.control_type() {
        Empty       => "flo-empty",
        Container   => "flo-container",
        Button      => "flo-button",
        Label       => "flo-label",
        Canvas      => "flo-canvas",
        Slider      => "flo-slider"
    }
}

impl ToHtml for Control {
    fn to_html_subcomponent(&self, base_path: &str, controller_path: &str) -> DomNode {
        // Start with the main element
        let mut result = DomElement::new(control_class(self));

        // The base path changes when the controller changes
        let new_path;
        let mut subcomponent_path   = controller_path;

        if let Some(subcontroller_name) = self.controller() {
            new_path                = format!("{}/{}", controller_path, utf8_percent_encode(subcontroller_name, DEFAULT_ENCODE_SET));
            subcomponent_path       = &new_path;
        }

        // Add any subcomponents or text for this control
        for attribute in self.attributes() {
            result.append_child_node(attribute.to_html_subcomponent(base_path, subcomponent_path));
        }

        // Flatten to create a 'clean' DOM without collections or empty nodes
        result.flatten();
        result
    }
}

impl ToHtml for ControlAttribute {
    fn to_html_subcomponent(&self, base_path: &str, controller_path: &str) -> DomNode {
        use ui::ControlAttribute::*;

        match self {
            &SubComponents(ref subcomponents) => {
                let mut result = DomCollection::new(vec![]);

                // Subcomponents go inside the div
                let subcomponent_nodes = subcomponents.iter()
                    .map(|control| control.to_html_subcomponent(base_path, controller_path));
                
                for node in subcomponent_nodes { 
                    result.append_child_node(node);
                }

                result
            },

            &Text(ref text) => DomElement::new("div").with(vec![
                DomAttribute::new("class", "text"),
                DomText::new(&text.to_string())
            ]),

            &Canvas(ref canvas) => {
                // Use the canvas's name if it has one, otherwise the ID
                let canvas_name = {
                    if let Some(name) = canvas.name() {
                        name
                    } else {
                        canvas.id().to_string()
                    }
                };

                // Build the URL from the base path
                let canvas_url = format!("{}/c{}/{}", base_path, controller_path, utf8_percent_encode(&canvas_name, DEFAULT_ENCODE_SET));

                // We attach canvas details to the node when it has a canvas attached to it
                DomCollection::new(vec![
                    DomAttribute::new("flo-canvas",     &canvas_url),
                    DomAttribute::new("flo-name",       &canvas_name),
                    DomAttribute::new("flo-controller", if controller_path.len() > 0 { &controller_path[1..] } else { "" })
                ])
            }

            &AppearanceAttr(ref appearance) => appearance.to_html_subcomponent(base_path, controller_path),
            &FontAttr(ref font_attribute)   => font_attribute.to_html_subcomponent(base_path, controller_path),
            &StateAttr(ref state)           => state.to_html_subcomponent(base_path, controller_path),

            &BoundingBox(_) => DomEmpty::new(),
            &Id(_)          => DomEmpty::new(),
            &Controller(_)  => DomEmpty::new(),
            &Action(_, _)   => DomEmpty::new()
        }
    }
}

impl ToHtml for Appearance {
    fn to_html_subcomponent(&self, base_path: &str, controller_path: &str) -> DomNode {
        use ui::Appearance::*;

        match self {
            &Background(ref col) => {
                let (r, g, b, a)    = col.to_rgba();
                let (r, g, b)       = ((r*255.0).floor() as i32, (g*255.0).floor() as i32, (b*255.0).floor() as i32);

                DomAttribute::new("style", &format!("background-color: rgba({}, {}, {}, {});", r, g, b, a))
            },

            &Foreground(ref col) => {
                let (r, g, b, a)    = col.to_rgba();
                let (r, g, b)       = ((r*255.0).floor() as i32, (g*255.0).floor() as i32, (b*255.0).floor() as i32);

                DomAttribute::new("style", &format!("color: rgba({}, {}, {}, {});", r, g, b, a))
            },

            &Image(ref image) => {
                // Use the image's name if it has one, otherwise the ID
                let image_name = {
                    if let Some(name) = image.name() {
                        name
                    } else {
                        image.id().to_string()
                    }
                };

                // Build the URL from the base path
                let image_url = format!("{}/i{}/{}", base_path, controller_path, utf8_percent_encode(&image_name, DEFAULT_ENCODE_SET));

                // Style attribute to render this image as the background
                DomAttribute::new("style", &format!("background: no-repeat center/contain url('{}');", image_url))
            }
        }
    }
}

impl ToHtml for State {
    fn to_html_subcomponent(&self, _base_path: &str, _controller_path: &str) -> DomNode {
        DomEmpty::new()
    }
}

impl ToHtml for Font {
    fn to_html_subcomponent(&self, base_path: &str, controller_path: &str) -> DomNode {
        use ui::Font::*;

        match self {
            &Size(size)     => DomAttribute::new("style", &format!("font-size: {}px;", size)),
            &Align(align)   => align.to_html_subcomponent(base_path, controller_path)
        }
    }
}

impl ToHtml for TextAlign {
    fn to_html_subcomponent(&self, _base_path: &str, _controller_path: &str) -> DomNode {
        use ui::TextAlign::*;

        match self {
            &Left   => DomAttribute::new("style", &format!("text-align: left;")),
            &Center => DomAttribute::new("style", &format!("text-align: center;")),
            &Right  => DomAttribute::new("style", &format!("text-align: right;"))
        }
    }
}

#[cfg(test)]
mod test {
    use canvas::*;
    use super::*;
    use std::sync::*;

    #[test]
    fn can_convert_button_to_html() {
        assert!(Control::button().to_html("").to_string() == "<flo-button></flo-button>")
    }

    #[test]
    fn can_convert_label_to_html() {
        assert!(Control::label().with("Hello & goodbye").to_html("").to_string() == "<flo-label><div class=\"text\">Hello &amp; goodbye</div></flo-label>")
    }

    #[test]
    fn can_combine_style_attributes() {
        let ctrl = Control::label()
            .with(Appearance::Foreground(Color::Rgba(1.0, 0.0, 0.0, 1.0)))
            .with(Appearance::Background(Color::Rgba(0.0, 1.0, 0.0, 1.0)));

        assert!(ctrl.to_html("").to_string() == "<flo-label style=\"color: rgba(255, 0, 0, 1); background-color: rgba(0, 255, 0, 1);\"></flo-label>");
    }

    #[test]
    fn can_convert_container_to_html() {
        assert!(Control::container().with(vec![Control::button()]).to_html("").to_string() == "<flo-container><flo-button></flo-button></flo-container>")
    }

    #[test]
    fn can_convert_canvas_to_html() {
        let resource_manager    = ResourceManager::new();
        let canvas              = resource_manager.register(BindingCanvas::new());
        resource_manager.assign_name(&canvas, "test_canvas");

        let control = Control::canvas().with(canvas.clone());

        assert!(control.to_html("test/base").to_string() == "<flo-canvas flo-canvas=\"test/base/c/test_canvas\" flo-name=\"test_canvas\" flo-controller=\"\"></flo-canvas>")
    }

    #[test]
    fn canvas_includes_controller() {
        let resource_manager    = ResourceManager::new();
        let canvas              = resource_manager.register(BindingCanvas::new());
        resource_manager.assign_name(&canvas, "test_canvas");

        let control = Control::empty()
            .with_controller("Test")
            .with(vec![Control::canvas().with(canvas.clone())]);

        assert!(control.to_html("test/base").to_string() == "<flo-empty><flo-canvas flo-canvas=\"test/base/c/Test/test_canvas\" flo-name=\"test_canvas\" flo-controller=\"Test\"></flo-canvas></flo-empty>");
    }

    #[test]
    fn image_includes_controller() {
        let resource_manager    = ResourceManager::new();
        let image               = resource_manager.register(Image::Png(Arc::new(InMemoryImageData::new(vec![]))));
        resource_manager.assign_name(&image, "test_image");

        let control = Control::empty()
            .with_controller("Test")
            .with(vec![Control::empty().with(image)]);

        assert!(control.to_html("test/base").to_string() == "<flo-empty><flo-empty style=\"background: no-repeat center/contain url(&quot;test/base/i/Test/test_image&quot;);\"></flo-empty></flo-empty>");
    }
}
