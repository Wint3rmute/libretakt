use crate::frontend::app_state::ApplicationState;
use crate::frontend::sequencer::LocalSequencerState;
use crate::shared::ClientCommand;

/// Everything a view function needs to render and queue commands,
/// deliberately excluding navigation state (`State`) so that routing
/// logic stays in `LibretaktUI::update` and views stay pure renderers.
pub struct ViewCtx<'vctx> {
    pub app_state: &'vctx ApplicationState,
    pub outbox: &'vctx mut Vec<ClientCommand>,
    pub track_params: &'vctx mut Vec<[f32; 4]>,
    pub local_seq: &'vctx LocalSequencerState,
}
