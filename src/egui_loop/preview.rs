use crate::app_state::SharedAppState;
use crate::ui::preview::PreviewPanel;
use crate::ui::properties::PropertiesPanel;
use crate::ui::timeline::TimelineWindow;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub(super) enum AppWakeEvent {
    ContinueLoop,
}

pub struct RegisteredPreview {
    pub panel: Rc<RefCell<PreviewPanel>>,
    pub dialogs: Rc<RefCell<crate::ui::dialogs::DialogSet>>,
    pub timeline: Rc<RefCell<TimelineWindow>>,
    pub properties: Rc<RefCell<PropertiesPanel>>,
    pub state: SharedAppState,
}

pub type PreviewSlot = Rc<RefCell<Option<RegisteredPreview>>>;

pub fn make_preview_slot() -> PreviewSlot {
    Rc::new(RefCell::new(None))
}

pub fn set_preview(
    slot: &PreviewSlot,
    panel: Rc<RefCell<PreviewPanel>>,
    dialogs: Rc<RefCell<crate::ui::dialogs::DialogSet>>,
    timeline: Rc<RefCell<TimelineWindow>>,
    properties: Rc<RefCell<PropertiesPanel>>,
    state: SharedAppState,
) {
    *slot.borrow_mut() = Some(RegisteredPreview {
        panel,
        dialogs,
        timeline,
        properties,
        state,
    });
}
