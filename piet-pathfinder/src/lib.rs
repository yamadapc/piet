use std::borrow::Cow;
use std::convert::TryInto;
use std::ops::RangeBounds;
use std::sync::{Arc, Mutex};

use pathfinder_canvas::{
    CanvasFontContext, FillStyle, ImageSmoothingQuality, Transform2F, Vector2F, Vector2I,
};
use pathfinder_content::pattern::Pattern;
use pathfinder_renderer::scene::RenderTarget;
use skribo::FontCollection;

use font_kit::error::{FontLoadingError, SelectionError};
use font_kit::family_handle::FamilyHandle;
use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::Properties;
use font_kit::source::Source;
use pathfinder_color::ColorU;
use piet::kurbo::{Affine, Line, PathEl, Point, Rect, Shape, Size};
use piet::{
    Color, Error, FixedGradient, FontFamily, FontFamilyInner, HitTestPoint, HitTestPosition,
    ImageFormat, InterpolationMode, IntoBrush, LineMetric, RenderContext, StrokeStyle,
    TextAlignment, TextAttribute, TextLayout, TextStorage,
};
use std::any::Any;

static TOLERANCE: f64 = 0.1;

pub struct PathFinderRenderContext<'a> {
    canvas: &'a mut pathfinder_canvas::CanvasRenderingContext2D,
    text: Text,
}

impl<'a> PathFinderRenderContext<'a> {
    pub fn new(
        canvas: &'a mut pathfinder_canvas::CanvasRenderingContext2D,
        font_source: Arc<FontSource>,
    ) -> Self {
        PathFinderRenderContext {
            canvas,
            text: Text { font_source },
        }
    }
}

#[derive(Clone)]
pub enum Brush {
    Solid(u32),
    Gradient,
}

impl IntoBrush<PathFinderRenderContext<'_>> for Brush {
    fn make_brush<'b>(
        &'b self,
        _piet: &mut PathFinderRenderContext,
        _bbox: impl FnOnce() -> Rect,
    ) -> std::borrow::Cow<'b, Brush> {
        Cow::Borrowed(self)
    }
}

#[derive(Clone)]
pub struct Text {
    font_source: Arc<FontSource>,
}

impl piet::Text for Text {
    type TextLayoutBuilder = TextLayoutBuilder;
    type TextLayout = PathfinderTextLayout;

    fn font_family(&mut self, family_name: &str) -> Option<FontFamily> {
        let family = self.font_source.select_family_by_name(family_name);
        family
            .ok()
            .map(|_family| FontFamily::new_unchecked(family_name))
    }

    fn load_font(&mut self, data: &[u8]) -> Result<FontFamily, Error> {
        let font_handle = font_kit::handle::Handle::from_memory(Arc::new(data.to_owned()), 0);
        let font = self
            .font_source
            .in_memory_source
            .lock()
            .unwrap()
            .add_font(font_handle)
            .map_err(|err| match err {
                FontLoadingError::NoSuchFontInCollection => Error::MissingFont,
                FontLoadingError::UnknownFormat => Error::FontLoadingFailed,
                FontLoadingError::Parse => Error::FontLoadingFailed,
                _ => Error::BackendError(Box::new(err)),
            })?;
        Ok(FontFamily::new_unchecked(font.family_name()))
    }

    fn new_text_layout(&mut self, text: impl TextStorage) -> Self::TextLayoutBuilder {
        TextLayoutBuilder {
            text: std::rc::Rc::new(text),
        }
    }
}

pub struct TextLayoutBuilder {
    text: std::rc::Rc<dyn TextStorage>,
}

impl piet::TextLayoutBuilder for TextLayoutBuilder {
    type Out = PathfinderTextLayout;

    fn max_width(self, width: f64) -> Self {
        self
    }

    fn alignment(self, alignment: TextAlignment) -> Self {
        self
    }

    fn default_attribute(self, attribute: impl Into<TextAttribute>) -> Self {
        self
    }

    fn range_attribute(
        self,
        range: impl RangeBounds<usize>,
        attribute: impl Into<TextAttribute>,
    ) -> Self {
        self
    }

    fn build(self) -> Result<Self::Out, Error> {
        Ok(PathfinderTextLayout {
            size: Default::default(),
            inner: self.text,
        })
    }
}

#[derive(Clone)]
pub struct PathfinderTextLayout {
    size: Size,
    inner: std::rc::Rc<dyn TextStorage>,
}

impl TextLayout for PathfinderTextLayout {
    fn size(&self) -> Size {
        // todo!()
        self.size
    }

    fn trailing_whitespace_width(&self) -> f64 {
        // todo!()
        0.0
    }

    fn image_bounds(&self) -> Rect {
        // todo!()
        Default::default()
    }

    fn text(&self) -> &str {
        // todo!()
        self.inner.as_str()
    }

    fn line_text(&self, line_number: usize) -> Option<&str> {
        // todo!()
        None
    }

    fn line_metric(&self, line_number: usize) -> Option<LineMetric> {
        // todo!()
        None
    }

    fn line_count(&self) -> usize {
        // todo!()
        0
    }

    fn hit_test_point(&self, point: Point) -> HitTestPoint {
        // todo!()
        HitTestPoint::default()
    }

    fn hit_test_text_position(&self, idx: usize) -> HitTestPosition {
        // todo!()
        HitTestPosition::default()
    }
}

#[derive(Clone)]
pub struct Image {
    inner: image::RgbaImage,
}

impl piet::Image for Image {
    fn size(&self) -> Size {
        let (width, height) = self.inner.dimensions();
        Size::new(width as f64, height as f64)
    }
}

impl pathfinder_canvas::CanvasImageSource for Image {
    fn to_pattern(
        self,
        dest_context: &mut pathfinder_canvas::CanvasRenderingContext2D,
        transform: pathfinder_canvas::Transform2F,
    ) -> Pattern {
        let mut p = Pattern::from_image(pathfinder_content::pattern::Image::from_image_buffer(
            self.inner,
        ));
        p.apply_transform(transform);
        p
    }
}

impl<'a> RenderContext for PathFinderRenderContext<'a> {
    type Brush = Brush;
    type Text = Text;
    type TextLayout = PathfinderTextLayout;
    type Image = Image;

    fn status(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn solid_brush(&mut self, color: Color) -> Self::Brush {
        Brush::Solid(color.as_rgba_u32())
    }

    fn gradient(&mut self, gradient: impl Into<FixedGradient>) -> Result<Self::Brush, Error> {
        let gradient = gradient.into();
        Ok(Brush::Gradient)
        // todo!()
    }

    fn clear(&mut self, region: impl Into<Option<Rect>>, color: Color) {
        let region = region.into();
        if let Some(rect) = region {
            self.canvas
                .set_fill_style(FillStyle::Color(ColorU::from_u32(color.as_rgba_u32())));
            self.canvas.fill_rect(pathfinder_geometry::rect::RectF::new(
                vec2f_from_point(rect.origin()),
                vec2f_from_size(rect.size()),
            ));
        } else {
            self.canvas.clear();
        }
    }

    fn stroke(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>, width: f64) {
        self.set_fill_style(&shape, brush);
        self.canvas.set_line_width(width as f32);
        self.canvas.stroke_path(path2d_from_shape(shape))
    }

    fn stroke_styled(
        &mut self,
        shape: impl Shape,
        brush: &impl IntoBrush<Self>,
        width: f64,
        style: &StrokeStyle,
    ) {
        self.canvas.set_line_width(width as f32);
        self.canvas.stroke_path(path2d_from_shape(shape))
        // todo!()
    }

    fn fill(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {
        self.set_fill_style(&shape, brush);
        self.canvas.fill_path(
            path2d_from_shape(shape),
            pathfinder_canvas::FillRule::Winding,
        );
    }

    fn fill_even_odd(&mut self, shape: impl Shape, brush: &impl IntoBrush<Self>) {
        self.set_fill_style(&shape, brush);
        self.canvas.fill_path(
            path2d_from_shape(shape),
            pathfinder_canvas::FillRule::EvenOdd,
        );
    }

    fn clip(&mut self, shape: impl Shape) {
        self.canvas.clip_path(
            path2d_from_shape(shape),
            pathfinder_canvas::FillRule::Winding,
        )
    }

    fn text(&mut self) -> &mut Self::Text {
        // self.canvas.font()
        // todo!()
        &mut self.text
    }

    fn draw_text(&mut self, layout: &Self::TextLayout, pos: impl Into<Point>) {
        // todo!()
        self.canvas
            .fill_text(layout.text(), vec2f_from_point(pos.into()));
    }

    fn save(&mut self) -> Result<(), Error> {
        self.canvas.save();
        Ok(())
    }

    fn restore(&mut self) -> Result<(), Error> {
        self.canvas.restore();
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn transform(&mut self, transform: Affine) {
        let transform = transform.as_coeffs();
        let ptransform = pathfinder_canvas::Transform2F::row_major(
            transform[0] as f32,
            transform[2] as f32,
            transform[1] as f32,
            transform[3] as f32,
            transform[4] as f32,
            transform[5] as f32,
        );
        self.canvas.set_transform(&ptransform)
    }

    fn make_image(
        &mut self,
        width: usize,
        height: usize,
        buf: &[u8],
        format: ImageFormat,
    ) -> Result<Self::Image, Error> {
        match format {
            ImageFormat::RgbaSeparate => Ok(Image {
                inner: image::RgbaImage::from_raw(
                    width
                        .try_into()
                        .ok()
                        .ok_or_else(|| piet::Error::NotSupported)?,
                    height
                        .try_into()
                        .ok()
                        .ok_or_else(|| piet::Error::NotSupported)?,
                    buf.to_owned(),
                )
                .ok_or_else(|| piet::Error::InvalidInput)?,
            }),
            _ => Err(piet::Error::NotSupported),
        }
    }

    fn draw_image(
        &mut self,
        image: &Self::Image,
        dst_rect: impl Into<Rect>,
        interp: InterpolationMode,
    ) {
        self.set_interpolation(interp);
        self.canvas
            .draw_image((*image).clone(), rectf_from_rect(dst_rect.into()));
    }

    fn draw_image_area(
        &mut self,
        image: &Self::Image,
        src_rect: impl Into<Rect>,
        dst_rect: impl Into<Rect>,
        interp: InterpolationMode,
    ) {
        self.set_interpolation(interp);
        self.canvas.draw_subimage(
            (*image).clone(),
            rectf_from_rect(src_rect.into()),
            rectf_from_rect(dst_rect.into()),
        );
    }

    fn capture_image_area(&mut self, _src_rect: impl Into<Rect>) -> Result<Self::Image, Error> {
        Err(Error::NotSupported)
    }

    fn blurred_rect(&mut self, rect: Rect, blur_radius: f64, brush: &impl IntoBrush<Self>) {
        let size = piet::util::size_for_blurred_rect(rect, blur_radius);
        let width = size.width as usize;
        let height = size.height as usize;
        let mut data = vec![0u8; width * height];
        let rect_exp = piet::util::compute_blurred_rect(rect, blur_radius, width, &mut data);
        let maybe_buffer = image::RgbaImage::from_raw(width as u32, height as u32, data);
        if let Some(buffer) = maybe_buffer {
            let image = Self::Image { inner: buffer };
            self.canvas
                .draw_image(image, vec2f_from_point(rect_exp.origin()));
        }
    }

    fn current_transform(&self) -> Affine {
        let t = self.canvas.transform();
        Affine::new([
            t.matrix.m11().into(),
            t.matrix.m21().into(),
            t.matrix.m12().into(),
            t.matrix.m22().into(),
            t.vector.x().into(),
            t.vector.y().into(),
        ])
    }
}

impl<'a> PathFinderRenderContext<'a> {
    fn set_interpolation(&mut self, interp: InterpolationMode) {
        use InterpolationMode::*;
        match interp {
            NearestNeighbor => self.canvas.set_image_smoothing_enabled(false),
            Bilinear => {
                self.canvas.set_image_smoothing_enabled(true);
                // I'm assuming that the lowest quality is bi-linear.
                self.canvas
                    .set_image_smoothing_quality(ImageSmoothingQuality::Low);
            }
        }
    }
}

fn path2d_from_shape(shape: impl Shape) -> pathfinder_canvas::Path2D {
    let mut path = pathfinder_canvas::Path2D::new();
    if let Some(Line { p0, p1 }) = shape.as_line() {
        path.move_to(vec2f_from_point(p0));
        path.line_to(vec2f_from_point(p1));
    } else if let Some(rect) = shape.as_rect() {
        path.rect(rectf_from_rect(rect));
    } else if let Some(els) = shape.as_path_slice() {
        for element in els {
            apply_el(&mut path, *element);
        }
    } else {
        let bez_path = shape.path_elements(TOLERANCE);
        for element in bez_path {
            apply_el(&mut path, element);
        }
    }
    path
}

fn apply_el(path: &mut pathfinder_canvas::Path2D, element: PathEl) {
    match element {
        PathEl::MoveTo(point) => {
            path.move_to(vec2f_from_point(point));
        }
        PathEl::LineTo(point) => {
            path.line_to(vec2f_from_point(point));
        }
        PathEl::QuadTo(ctrl_point, to_point) => {
            path.quadratic_curve_to(vec2f_from_point(ctrl_point), vec2f_from_point(to_point));
        }
        PathEl::CurveTo(ctrl0_point, ctrl1_point, to_point) => {
            let ctrl0 = vec2f_from_point(ctrl0_point);
            let ctrl1 = vec2f_from_point(ctrl1_point);
            let to = vec2f_from_point(to_point);
            path.bezier_curve_to(ctrl0, ctrl1, to);
        }
        PathEl::ClosePath => {
            path.close_path();
        }
    }
}

fn vec2f_from_point(point: Point) -> Vector2F {
    pathfinder_geometry::vector::vec2f(point.x as f32, point.y as f32)
}

fn vec2f_from_size(size: Size) -> Vector2F {
    pathfinder_geometry::vector::vec2f(size.width as f32, size.height as f32)
}

fn vec2i_from_size(size: Size) -> Vector2I {
    pathfinder_geometry::vector::vec2i(size.width as i32, size.height as i32)
}

fn rectf_from_rect(rect: Rect) -> pathfinder_geometry::rect::RectF {
    let origin = vec2f_from_point(rect.origin());
    let size = vec2f_from_size(rect.size());
    pathfinder_geometry::rect::RectF::new(origin, size)
}

pub struct FontSource {
    in_memory_source: std::sync::Mutex<font_kit::sources::mem::MemSource>,
    multi_source: font_kit::sources::multi::MultiSource,
}

impl FontSource {
    pub fn new(sources: Vec<Box<dyn font_kit::source::Source>>) -> Self {
        FontSource {
            multi_source: font_kit::sources::multi::MultiSource::from_sources(sources),
            in_memory_source: Mutex::new(font_kit::sources::mem::MemSource::empty()),
        }
    }
}

impl font_kit::source::Source for FontSource {
    fn all_fonts(&self) -> Result<Vec<Handle>, SelectionError> {
        let mut handles = self.multi_source.all_fonts()?;
        handles.extend(
            self.in_memory_source
                .lock()
                .unwrap()
                .all_fonts()?
                .into_iter(),
        );
        Ok(handles)
    }

    fn all_families(&self) -> Result<Vec<String>, SelectionError> {
        let mut handles = self.multi_source.all_families()?;
        handles.extend(
            self.in_memory_source
                .lock()
                .unwrap()
                .all_families()?
                .into_iter(),
        );
        Ok(handles)
    }

    fn select_family_by_name(&self, family_name: &str) -> Result<FamilyHandle, SelectionError> {
        if let Ok(handle) = self
            .in_memory_source
            .lock()
            .unwrap()
            .select_family_by_name(family_name)
        {
            Ok(handle)
        } else {
            self.multi_source.select_family_by_name(family_name)
        }
    }

    fn select_by_postscript_name(&self, postscript_name: &str) -> Result<Handle, SelectionError> {
        if let Ok(handle) = self
            .in_memory_source
            .lock()
            .unwrap()
            .select_by_postscript_name(postscript_name)
        {
            Ok(handle)
        } else {
            self.multi_source.select_by_postscript_name(postscript_name)
        }
    }

    fn select_family_by_generic_name(
        &self,
        family_name: &FamilyName,
    ) -> Result<FamilyHandle, SelectionError> {
        if let Ok(handle) = self
            .in_memory_source
            .lock()
            .unwrap()
            .select_family_by_generic_name(family_name)
        {
            Ok(handle)
        } else {
            self.multi_source.select_family_by_generic_name(family_name)
        }
    }

    fn select_best_match(
        &self,
        family_names: &[FamilyName],
        properties: &Properties,
    ) -> Result<Handle, SelectionError> {
        if let Ok(handle) = self
            .in_memory_source
            .lock()
            .unwrap()
            .select_best_match(family_names, properties)
        {
            Ok(handle)
        } else {
            self.multi_source
                .select_best_match(family_names, properties)
        }
    }

    fn select_descriptions_in_family(
        &self,
        family: &FamilyHandle,
    ) -> Result<Vec<Properties>, SelectionError> {
        if let Ok(properties) = self
            .in_memory_source
            .lock()
            .unwrap()
            .select_descriptions_in_family(family)
        {
            Ok(properties)
        } else {
            self.multi_source.select_descriptions_in_family(family)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<'a> PathFinderRenderContext<'a> {
    fn set_fill_style(&mut self, shape: &impl Shape, brush: &impl IntoBrush<Self>) {
        let brush = brush.make_brush(self, || shape.bounding_box());
        match *brush {
            Brush::Solid(color) => self
                .canvas
                .set_fill_style(FillStyle::Color(ColorU::from_u32(color))),
            Brush::Gradient => {}
        }
    }
}
