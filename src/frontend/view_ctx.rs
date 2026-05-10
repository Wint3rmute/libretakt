use crate::frontend::app_state::ApplicationState;
use crate::frontend::notifications::NotificationQueue;
use crate::frontend::sequencer::LocalSequencerState;
use crate::shared::ClientCommand;

/// Everything a view function needs to render and queue commands,
/// deliberately excluding navigation state (`State`) so that routing
/// logic stays in `LibretaktUI::update` and views stay pure renderers.
pub struct ViewCtx<'a> {
    pub app_state: &'a ApplicationState,
    pub outbox: &'a mut Vec<ClientCommand>,
    pub notifications: &'a mut NotificationQueue,
    pub track_params: &'a mut Vec<[f32; 4]>,
    pub local_seq: &'a LocalSequencerState,
}
