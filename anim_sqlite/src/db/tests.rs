use super::*;
use super::db_enum::*;
use super::flo_store::*;
use super::flo_query::*;
use super::motion_path_type::*;

use animation;
use animation::LayerEdit::*;
use animation::PaintEdit::*;

use std::time::Duration;

#[test]
fn can_create_new_database() {
    let db = AnimationDb::new();
    assert!(db.retrieve_and_clear_error().is_none());
}

fn core() -> AnimationDbCore<FloSqlite> {
    let connection = Connection::open_in_memory().unwrap();
    FloSqlite::setup(&connection).unwrap();

    let core = AnimationDbCore::new(connection);
    core
}

#[test]
fn initial_length_is_two_minutes() {
    assert!(core().db.query_duration().unwrap() == Duration::from_secs(120));
}

#[test]
fn initial_frame_rate_is_30fps() {
    assert!(core().db.query_frame_length().unwrap() == Duration::new(0, 33_333_333));
}

#[test]
fn insert_set_size() {
    core().insert_edits(&[AnimationEdit::SetSize(1980.0, 1080.0)]).unwrap();
}

#[test]
fn insert_add_new_layer() {
    core().insert_edits(&[AnimationEdit::AddNewLayer(24)]).unwrap();
}

#[test]
fn remove_layer() {
    core().insert_edits(&[AnimationEdit::RemoveLayer(24)]).unwrap();
}

#[test]
fn add_key_frame() {
    core().insert_edits(&[AnimationEdit::Layer(24, AddKeyFrame(Duration::from_millis(300)))]).unwrap();
}

#[test]
fn remove_key_frame() {
    core().insert_edits(&[AnimationEdit::Layer(24, RemoveKeyFrame(Duration::from_millis(300)))]).unwrap();
}

#[test]
fn select_brush() {
    core().insert_edits(&[AnimationEdit::Layer(24, 
        Paint(Duration::from_millis(300), 
            SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()), 
                BrushDrawingStyle::Draw
            )
        )
    )]).unwrap();
}

#[test]
fn brush_properties() {
    core().insert_edits(&[AnimationEdit::Layer(24,
        Paint(Duration::from_millis(300),
            BrushProperties(ElementId::Unassigned, animation::BrushProperties::new())
        )
    )]).unwrap();
}

#[test]
fn brush_stroke() {
    core().insert_edits(&[AnimationEdit::Layer(24,
        Paint(Duration::from_millis(300),
            BrushStroke(ElementId::Unassigned, Arc::new(vec![
                RawPoint::from((0.0, 0.0)),
                RawPoint::from((10.0, 0.0)),
                RawPoint::from((10.0, 10.0)),
                RawPoint::from((0.0, 10.0)),
                RawPoint::from((0.0, 0.0))
            ]))
        )
    )]).unwrap();
}

#[test]
fn translate_motion() {
    let start_point = TimePoint::new(10.0, 20.0, Duration::from_millis(0));
    let end_point   = TimePoint::new(500.0, 400.0, Duration::from_millis(2000));

    core().insert_edits(&[
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::Create),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::SetType(MotionType::Translate)),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::SetOrigin(30.0, 40.0)),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::SetPath(TimeCurve::new(start_point, end_point))),
    ]).unwrap();
}

fn test_updates(updates: Vec<DatabaseUpdate>) {
    let core    = core();
    let mut db  = core.db;

    db.update(updates).unwrap();

    assert!(db.stack_is_empty());
}

#[test]
fn smoke_update_canvas_size() {
    test_updates(vec![DatabaseUpdate::UpdateCanvasSize(100.0, 200.0)])
}

#[test]
fn smoke_push_edit_type() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame), 
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_motion_origin() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::MotionSetOrigin), 
        DatabaseUpdate::PushEditLogElementId(1), 
        DatabaseUpdate::PushEditLogMotionOrigin(42.0, 24.0),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_motion_type_translate() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::MotionSetOrigin), 
        DatabaseUpdate::PushEditLogElementId(1), 
        DatabaseUpdate::PushEditLogMotionType(MotionType::Translate),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_motion_element() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::MotionSetOrigin), 
        DatabaseUpdate::PushEditLogElementId(1), 
        DatabaseUpdate::PushEditLogMotionElement(2),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_motion_path() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::MotionSetOrigin), 
        DatabaseUpdate::PushEditLogElementId(1), 
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushEditLogMotionPath(4),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn adding_edit_type_increases_log_length() {
    let core    = core();
    let mut db  = core.db;

    assert!(db.query_edit_log_length().unwrap() == 0);

    db.update(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame), 
        DatabaseUpdate::Pop
    ]).unwrap();

    assert!(db.query_edit_log_length().unwrap() == 1);
}

#[test]
fn can_query_edit_type() {
    let core    = core();
    let mut db  = core.db;

    assert!(db.query_edit_log_length().unwrap() == 0);

    db.update(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame), 
        DatabaseUpdate::PushEditLogLayer(3),
        DatabaseUpdate::Pop,
        DatabaseUpdate::PushEditType(EditLogType::SetSize),
        DatabaseUpdate::Pop,
    ]).unwrap();

    let edit_entries = db.query_edit_log_values(0, 1).unwrap();
    assert!(edit_entries.len() == 1);
    assert!(edit_entries[0].edit_type == EditLogType::LayerAddKeyFrame);
    assert!(edit_entries[0].layer_id == Some(3));
    assert!(edit_entries[0].when.is_none());
    assert!(edit_entries[0].brush.is_none());
    assert!(edit_entries[0].brush_properties_id.is_none());

    let edit_entries2 = db.query_edit_log_values(1, 2).unwrap();
    assert!(edit_entries2.len() == 1);
    assert!(edit_entries2[0].edit_type == EditLogType::SetSize);

    let edit_entries3 = db.query_edit_log_values(2, 3).unwrap();
    assert!(edit_entries3.len() == 0);

    let edit_entries4 = db.query_edit_log_values(0, 2).unwrap();
    assert!(edit_entries4.len() == 2);
}

#[test]
fn smoke_pop_edit_log_set_size() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame), 
        DatabaseUpdate::PopEditLogSetSize(100.0, 200.0)
    ]);
}

#[test]
fn smoke_push_edit_log_layer() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame), 
        DatabaseUpdate::PushEditLogLayer(1),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_edit_log_when() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame), 
        DatabaseUpdate::PushEditLogWhen(Duration::from_millis(2000)),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_edit_log_raw_points() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame), 
        DatabaseUpdate::PushRawPoints(Arc::new(vec![RawPoint::from((0.0, 0.0)), RawPoint::from((1.0, 2.0))])),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_brush_type() {
    test_updates(vec![
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_ink_brush() {
    test_updates(vec![
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::PushInkBrush(1.0, 2.0, 3.0),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_color_type() {
    test_updates(vec![
        DatabaseUpdate::PushColorType(ColorType::Rgb),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_color_rgb() {
    test_updates(vec![
        DatabaseUpdate::PushColorType(ColorType::Rgb),
        DatabaseUpdate::PushRgb(1.0, 1.0, 1.0),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_color_hsluv() {
    test_updates(vec![
        DatabaseUpdate::PushColorType(ColorType::Hsluv),
        DatabaseUpdate::PushHsluv(20.0, 100.0, 50.0),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_brush_properties() {
    test_updates(vec![
        DatabaseUpdate::PushColorType(ColorType::Hsluv),
        DatabaseUpdate::PushHsluv(20.0, 100.0, 50.0),
        DatabaseUpdate::PushBrushProperties(100.0, 1.0),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_editlog_brush_properties() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerPaintBrushProperties),
        DatabaseUpdate::PushColorType(ColorType::Hsluv),
        DatabaseUpdate::PushHsluv(20.0, 100.0, 50.0),
        DatabaseUpdate::PushBrushProperties(100.0, 1.0),
        DatabaseUpdate::PopEditLogBrushProperties
    ])
}

#[test]
fn smoke_editlog_element_id() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerPaintSelectBrush),
        DatabaseUpdate::PushEditLogElementId(3),
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::PushInkBrush(1.0, 2.0, 3.0),
        DatabaseUpdate::PopEditLogBrush(DrawingStyleType::Erase)
    ])
}

#[test]
fn smoke_editlog_brush() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerPaintSelectBrush),
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::PushInkBrush(1.0, 2.0, 3.0),
        DatabaseUpdate::PopEditLogBrush(DrawingStyleType::Erase)
    ])
}

#[test]
fn smoke_layer_type() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_delete_layer() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PopDeleteLayer
    ])
}

#[test]
fn smoke_assign_layer() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopDeleteLayer
    ])
}

#[test]
fn smoke_layer_for_assigned_id() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::Pop,
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PopDeleteLayer
    ])
}

#[test]
fn smoke_add_key_frame() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000))
    ])
}

#[test]
fn smoke_remove_key_frame() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PopRemoveKeyFrame(Duration::from_millis(2000))
    ])
}

#[test]
fn smoke_push_nearest_keyframe() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_vector_element_type() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke, Duration::from_millis(2500)),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_vector_element_assign_id() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke, Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(42),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_pop_vector_brush_element() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushDefinition, Duration::from_millis(2500)),
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::PushInkBrush(1.0, 2.0, 3.0),
        DatabaseUpdate::PopVectorBrushElement(DrawingStyleType::Draw),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_pop_vector_brush_properties_element() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushProperties, Duration::from_millis(2500)),
        DatabaseUpdate::PushColorType(ColorType::Hsluv),
        DatabaseUpdate::PushHsluv(20.0, 100.0, 50.0),
        DatabaseUpdate::PushBrushProperties(100.0, 1.0),
        DatabaseUpdate::PopVectorBrushPropertiesElement,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_pop_brush_points() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke, Duration::from_millis(2500)),
        DatabaseUpdate::PopBrushPoints(Arc::new(vec![BrushPoint { position: (10.0, 5.0), cp1: (20.0, 20.0), cp2: (30.0, 30.0), width: 10.0 }])),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_create_motion() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::CreateMotion(2)
    ])
}

#[test]
fn smoke_set_motion_type() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::SetMotionType(1, animation::MotionType::Translate)
    ])
}

#[test]
fn smoke_set_motion_origin() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::SetMotionOrigin(1, 20.0, 30.0)
    ])
}

#[test]
fn smoke_set_motion_path() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::SetMotionPath(1, MotionPathType::Position, 4)
    ])
}

#[test]
fn smoke_change_motion_path() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::SetMotionPath(1, MotionPathType::Position, 4),

        DatabaseUpdate::PushTimePoint(5.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(6.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(7.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(8.0, 2.0, 3.0),
        DatabaseUpdate::SetMotionPath(1, MotionPathType::Position, 4)
    ])
}

#[test]
fn smoke_attach_elements_to_motion() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke, Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(42),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,

        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::AddMotionAttachedElement(1, 42)
    ])
}
