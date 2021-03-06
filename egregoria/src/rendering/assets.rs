use geom::Color;
use imgui_inspect::InspectDragf;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Inspect)]
pub struct AssetID {
    pub id: u16,
}

impl AssetID {
    pub const CAR: AssetID = AssetID { id: 0 };
    pub const TRUCK: AssetID = AssetID { id: 1 };
}

#[derive(Copy, Clone, Serialize, Deserialize, Inspect)]
pub struct AssetRender {
    pub id: AssetID,
    pub hide: bool,
    #[inspect(proxy_type = "InspectDragf")]
    pub scale: f32,
    pub tint: Color,
    pub z: f32,
}
