use cctv_core::Aspect;
use gtk::{gdk, glib, graphene, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
mod imp {
    use super::*;
    pub struct Canvas {
        pub texture: RefCell<Option<gdk::Texture>>,
        pub zoom: Cell<f64>,
        pub pan: Cell<(f64, f64)>,
        pub aspect: Cell<Aspect>,
    }
    impl Default for Canvas {
        fn default() -> Self {
            Self {
                texture: RefCell::new(None),
                zoom: Cell::new(1.),
                pan: Cell::new((0., 0.)),
                aspect: Cell::new(Aspect::Fit),
            }
        }
    }
    #[glib::object_subclass]
    impl ObjectSubclass for Canvas {
        const NAME: &'static str = "CctvVideoCanvas";
        type Type = super::Canvas;
        type ParentType = gtk::Widget;
    }
    impl ObjectImpl for Canvas {}
    impl WidgetImpl for Canvas {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            if orientation == gtk::Orientation::Horizontal {
                (100, 480, -1, -1)
            } else {
                (80, 270, -1, -1)
            }
        }
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();
            let w = obj.width() as f32;
            let h = obj.height() as f32;
            snapshot.append_color(
                &gdk::RGBA::new(0.025, 0.03, 0.035, 1.),
                &graphene::Rect::new(0., 0., w, h),
            );
            let binding = self.texture.borrow();
            let Some(texture) = binding.as_ref() else {
                return;
            };
            let tw = texture.width() as f32;
            let th = texture.height() as f32;
            let factor = match self.aspect.get() {
                Aspect::Fit => (w / tw).min(h / th),
                Aspect::Fill => (w / tw).max(h / th),
                Aspect::Stretch => 1.,
            };
            let (basew, baseh) = if self.aspect.get() == Aspect::Stretch {
                (w, h)
            } else {
                (tw * factor, th * factor)
            };
            let dw = basew * self.zoom.get() as f32;
            let dh = baseh * self.zoom.get() as f32;
            let (px, py) = self.pan.get();
            let limit_x = ((dw - w) / 2.).max(0.);
            let limit_y = ((dh - h) / 2.).max(0.);
            let x = (w - dw) / 2. + (px as f32).clamp(-limit_x, limit_x);
            let y = (h - dh) / 2. + (py as f32).clamp(-limit_y, limit_y);
            snapshot.push_clip(&graphene::Rect::new(0., 0., w, h));
            snapshot.append_texture(texture, &graphene::Rect::new(x, y, dw, dh));
            snapshot.pop();
        }
    }
}
glib::wrapper! {pub struct Canvas(ObjectSubclass<imp::Canvas>) @extends gtk::Widget, @implements gtk::Accessible,gtk::Buildable,gtk::ConstraintTarget;}
impl Canvas {
    pub fn new(aspect: Aspect) -> Self {
        let canvas: Self = glib::Object::builder().build();
        canvas.imp().aspect.set(aspect);
        canvas.set_hexpand(true);
        canvas.set_vexpand(true);
        canvas.set_overflow(gtk::Overflow::Hidden);
        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let weak = canvas.downgrade();
        scroll.connect_scroll(move |_, _, dy| {
            if let Some(c) = weak.upgrade() {
                c.zoom_by(if dy < 0. { 1.15 } else { 1. / 1.15 });
            }
            glib::Propagation::Stop
        });
        canvas.add_controller(scroll);
        let drag = gtk::GestureDrag::new();
        let start = std::rc::Rc::new(Cell::new((0., 0.)));
        let weak = canvas.downgrade();
        let s = start.clone();
        drag.connect_drag_begin(move |_, _, _| {
            if let Some(c) = weak.upgrade() {
                s.set(c.imp().pan.get());
            }
        });
        let weak = canvas.downgrade();
        drag.connect_drag_update(move |_, dx, dy| {
            if let Some(c) = weak.upgrade() {
                let (x, y) = start.get();
                c.imp().pan.set((x + dx, y + dy));
                c.queue_draw();
            }
        });
        canvas.add_controller(drag);
        canvas
    }
    pub fn set_frame(&self, frame: &cctv_media::Frame) {
        let bytes = glib::Bytes::from_owned(frame.pixels.clone());
        let texture = gdk::MemoryTexture::new(
            frame.width,
            frame.height,
            gdk::MemoryFormat::B8g8r8a8,
            &bytes,
            frame.stride as usize,
        );
        *self.imp().texture.borrow_mut() = Some(texture.upcast());
        self.queue_draw();
    }
    pub fn zoom_by(&self, factor: f64) {
        self.imp()
            .zoom
            .set((self.imp().zoom.get() * factor).clamp(1., 8.));
        self.queue_draw();
    }
    pub fn reset(&self) {
        self.imp().zoom.set(1.);
        self.imp().pan.set((0., 0.));
        self.queue_draw();
    }
}
