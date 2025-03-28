use std::collections::BTreeMap;

use egui::{Context, Id, Pos2, Vec2b};

use super::{PlotBounds, PlotTransform};

/// Information about the plot that has to persist between frames.
#[derive(Clone)]
pub struct PlotMemory {
    /// Indicates if the plot uses automatic bounds.
    ///
    /// This is set to `false` whenever the user modifies
    /// the bounds, for example by moving or zooming.
    pub auto_bounds: Vec2b,

    /// Display string of the hovered legend item if any.
    pub hovered_legend_item: Option<String>,

    /// The transform from last frame.
    pub(crate) transform: PlotTransform,

    /// Allows to remember the first click position when performing a boxed zoom
    pub(crate) last_click_pos_for_zoom: Option<Pos2>,
    /// Allows to remember the first click position when performing a boxed zoom
    pub(crate) last_click_pos_for_spawn_search_zone: Option<Pos2>,

    /// The thickness of each of the axes the previous frame.
    ///
    /// This is used in the next frame to make the axes thicker
    /// in order to fit the labels, if necessary.
    pub(crate) x_axis_thickness: BTreeMap<usize, f32>,
    pub(crate) y_axis_thickness: BTreeMap<usize, f32>,
}

#[allow(dead_code)]
impl PlotMemory {
    #[inline]
    pub fn transform(&self) -> PlotTransform {
        self.transform
    }

    #[inline]
    pub fn set_transform(&mut self, t: PlotTransform) {
        self.transform = t;
    }

    /// Plot-space bounds.
    #[inline]
    pub fn bounds(&self) -> &PlotBounds {
        self.transform.bounds()
    }

    /// Plot-space bounds.
    #[inline]
    pub fn set_bounds(&mut self, bounds: PlotBounds) {
        self.transform.set_bounds(bounds);
    }
}

impl PlotMemory {
    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_temp(id))
    }

    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }
}
