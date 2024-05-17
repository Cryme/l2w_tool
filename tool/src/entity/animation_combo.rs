use crate::data::{AnimationComboId};
use crate::entity::{CommonEntity, GetEditParams};
use serde::{Deserialize, Serialize};

impl GetEditParams<()> for AnimationCombo {
    fn edit_params(&self) {}
}

impl CommonEntity<AnimationComboId> for AnimationCombo {
    fn name(&self) -> String {
        "".to_string()
    }

    fn desc(&self) -> String {
        "".to_string()
    }

    fn id(&self) -> AnimationComboId {
        self.id
    }

    fn changed(&self) -> bool {
        self._changed
    }

    fn deleted(&self) -> bool {
        self._deleted
    }

    fn new(id: AnimationComboId) -> Self {
        AnimationCombo {
            id,
            name: "New Anim".to_string(),
            anim_0: "".to_string(),
            anim_1: "".to_string(),
            anim_2: "".to_string(),
            loop_p: false,

            _changed: false,
            _deleted: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct AnimationCombo {
    pub id: AnimationComboId,
    pub name: String,
    pub anim_0: String,
    pub anim_1: String,
    pub anim_2: String,
    pub loop_p: bool,

    pub(crate) _changed: bool,
    pub(crate) _deleted: bool,
}
