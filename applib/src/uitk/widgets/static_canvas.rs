use crate::content::{ContentId, TrackedContent};
use crate::uitk::UiContext;
use crate::{Color, Rect};
use crate::{FbView, FbViewMut};

use super::dynamic_canvas::TileRenderer;

struct BufferCopyRenderer<'a, F: FbView> {
    src_fb: &'a TrackedContent<F>,
    fill_color: Color,
}

impl<'a, F1: FbView> TileRenderer for BufferCopyRenderer<'a, F1> {
    fn shape(&self) -> (u32, u32) {
        self.src_fb.as_ref().shape()
    }

    fn tile_shape(&self) -> (u32, u32) {
        self.shape()
    }

    fn content_id(&self, viewport_rect: &Rect) -> ContentId {
        ContentId::from_hash(&(self.src_fb.get_id(), viewport_rect))
    }

    fn render<F: FbViewMut>(&self, dst_fb: &mut F, viewport_rect: &Rect) {
        let src_fb = self.src_fb.as_ref().subregion(viewport_rect);
        dst_fb.fill(self.fill_color);
        dst_fb.copy_from_fb(&src_fb, (0, 0), false);
    }
}

impl<'a, F: FbViewMut> UiContext<'a, F> {
    pub fn static_canvas<F1: FbView>(
        &mut self,
        dst_rect: &Rect,
        src_fb: &TrackedContent<F1>,
        offsets: &mut (i64, i64),
        dragging: &mut (bool, bool),
        fill_color: Color,
    ) {
        let renderer = BufferCopyRenderer { src_fb, fill_color };

        self.dynamic_canvas(dst_rect, &renderer, offsets, dragging)
    }
}

pub fn set_autoscroll(dst_rect: &Rect, max_h: u32, offsets: &mut (i64, i64)) {
    let (_scroll_x0, scroll_y0) = offsets;
    *scroll_y0 = (max_h - dst_rect.h - 1).into();
}
