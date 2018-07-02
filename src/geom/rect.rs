use geom::{quad, scalar, Align, Edge, Quad, Range, Tri};
use math::num_traits::Float;
use math::{self, BaseNum, Point2, Vector2};
use std::ops::Neg;

/// Defines a Rectangle's bounds across the x and y axes.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Rect<S = scalar::Default> {
    /// The start and end positions of the Rectangle on the x axis.
    pub x: Range<S>,
    /// The start and end positions of the Rectangle on the y axis.
    pub y: Range<S>,
}

/// The distance between the inner edge of a border and the outer edge of the inner content.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Padding<S = scalar::Default> {
    /// Padding on the start and end of the *x* axis.
    pub x: Range<S>,
    /// Padding on the start and end of the *y* axis.
    pub y: Range<S>,
}

/// Either of the four corners of a **Rect**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Corner {
    /// The top left corner of a **Rect**.
    TopLeft,
    /// The top right corner of a **Rect**.
    TopRight,
    /// The bottom left corner of a **Rect**.
    BottomLeft,
    /// The bottom right corner of a **Rect**.
    BottomRight,
}

/// Yields even subdivisions of a `Rect`.
///
/// The four subdivisions will each be yielded as a `Rect` whose dimensions are exactly half of the
/// original `Rect`.
#[derive(Clone)]
pub struct Subdivisions<S = scalar::Default> {
    ranges: SubdivisionRanges<S>,
    subdivision_index: u8,
}

/// The ranges that describe the subdivisions of a `Rect`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct SubdivisionRanges<S = scalar::Default> {
    /// The first half of the x range.
    pub x_a: Range<S>,
    /// The second half of the x range.
    pub x_b: Range<S>,
    /// The first half of the y range.
    pub y_a: Range<S>,
    /// The second half of the y range.
    pub y_b: Range<S>,
}

/// An iterator yielding the four corners of a `Rect`.
#[derive(Clone, Debug)]
pub struct Corners<S = scalar::Default> {
    rect: Rect<S>,
    index: u8,
}

/// The triangles iterator yielded by the `Rect`.
pub type Triangles<S> = quad::Triangles<Point2<S>>;

/// The number of subdivisions when dividing a `Rect` in half along the *x* and *y* axes.
pub const NUM_SUBDIVISIONS: u8 = 4;

/// The number of subdivisions when dividing a `Rect` in half along the *x* and *y* axes.
pub const NUM_CORNERS: u8 = 4;

/// The number of triangles used to represent a `Rect`.
pub const NUM_TRIANGLES: u8 = 2;

impl<S> Padding<S>
where
    S: BaseNum,
{
    /// No padding.
    pub fn none() -> Self {
        Padding {
            x: Range::new(S::zero(), S::zero()),
            y: Range::new(S::zero(), S::zero()),
        }
    }
}

// Given some `SubdivisionRanges` and a subdivision index, produce the rect for that subdivision.
macro_rules! subdivision_from_index {
    ($ranges:expr,0) => {
        Rect {
            x: $ranges.x_a,
            y: $ranges.y_a,
        }
    };
    ($ranges:expr,1) => {
        Rect {
            x: $ranges.x_b,
            y: $ranges.y_a,
        }
    };
    ($ranges:expr,2) => {
        Rect {
            x: $ranges.x_a,
            y: $ranges.y_b,
        }
    };
    ($ranges:expr,3) => {
        Rect {
            x: $ranges.x_b,
            y: $ranges.y_b,
        }
    };
}

// Given some `Rect` and an index, produce the corner for that index.
macro_rules! corner_from_index {
    ($rect:expr,0) => {
        $rect.bottom_left()
    };
    ($rect:expr,1) => {
        $rect.bottom_right()
    };
    ($rect:expr,2) => {
        $rect.top_left()
    };
    ($rect:expr,3) => {
        $rect.top_right()
    };
}

impl<S> Rect<S>
where
    S: BaseNum,
{
    /// Construct a Rect from a given `Point` and `Dimensions`.
    pub fn from_xy_wh(p: Point2<S>, wh: Vector2<S>) -> Self {
        Rect {
            x: Range::from_pos_and_len(p.x, wh.x),
            y: Range::from_pos_and_len(p.y, wh.y),
        }
    }

    /// Construct a Rect from the given `x` `y` coordinates and `w` `h` dimensions.
    pub fn from_x_y_w_h(x: S, y: S, w: S, h: S) -> Self {
        Rect::from_xy_wh(Point2 { x, y }, Vector2 { x: w, y: h })
    }

    /// Construct a Rect at origin with the given dimensions.
    pub fn from_wh(wh: Vector2<S>) -> Self {
        let p = Point2 {
            x: S::zero(),
            y: S::zero(),
        };
        Self::from_xy_wh(p, wh)
    }

    /// Construct a Rect at origin with the given width and height.
    pub fn from_w_h(w: S, h: S) -> Self {
        Self::from_wh(Vector2 { x: w, y: h })
    }

    /// Construct a Rect from the coordinates of two points.
    pub fn from_corners(a: Point2<S>, b: Point2<S>) -> Self {
        let (left, right) = if a.x < b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (bottom, top) = if a.y < b.y { (a.y, b.y) } else { (b.y, a.y) };
        Rect {
            x: Range {
                start: left,
                end: right,
            },
            y: Range {
                start: bottom,
                end: top,
            },
        }
    }

    /// Converts `self` to an absolute `Rect` so that the magnitude of each range is always
    /// positive.
    pub fn absolute(self) -> Self {
        let x = self.x.absolute();
        let y = self.y.absolute();
        Rect { x, y }
    }

    /// The Rect representing the area in which two Rects overlap.
    pub fn overlap(self, other: Self) -> Option<Self> {
        self.x
            .overlap(other.x)
            .and_then(|x| self.y.overlap(other.y).map(|y| Rect { x: x, y: y }))
    }

    /// The Rect that encompass the two given sets of Rect.
    pub fn max(self, other: Self) -> Self
    where
        S: Float,
    {
        Rect {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    /// The position in the middle of the x bounds.
    pub fn x(&self) -> S {
        self.x.middle()
    }

    /// The position in the middle of the y bounds.
    pub fn y(&self) -> S {
        self.y.middle()
    }

    /// The xy position in the middle of the bounds.
    pub fn xy(&self) -> Point2<S> {
        [self.x(), self.y()].into()
    }

    /// The centered x and y coordinates as a tuple.
    pub fn x_y(&self) -> (S, S) {
        (self.x(), self.y())
    }

    /// The Rect's lowest y value.
    pub fn bottom(&self) -> S {
        self.y.absolute().start
    }

    /// The Rect's highest y value.
    pub fn top(&self) -> S {
        self.y.absolute().end
    }

    /// The Rect's lowest x value.
    pub fn left(&self) -> S {
        self.x.absolute().start
    }

    /// The Rect's highest x value.
    pub fn right(&self) -> S {
        self.x.absolute().end
    }

    /// The top left corner **Point**.
    pub fn top_left(&self) -> Point2<S> {
        [self.left(), self.top()].into()
    }

    /// The bottom left corner **Point**.
    pub fn bottom_left(&self) -> Point2<S> {
        [self.left(), self.bottom()].into()
    }

    /// The top right corner **Point**.
    pub fn top_right(&self) -> Point2<S> {
        [self.right(), self.top()].into()
    }

    /// The bottom right corner **Point**.
    pub fn bottom_right(&self) -> Point2<S> {
        [self.right(), self.bottom()].into()
    }

    /// The edges of the **Rect** in a tuple (top, bottom, left, right).
    pub fn l_r_b_t(&self) -> (S, S, S, S) {
        (self.left(), self.right(), self.bottom(), self.top())
    }

    /// Shift the Rect along the x axis.
    pub fn shift_x(self, x: S) -> Self {
        Rect {
            x: self.x.shift(x),
            ..self
        }
    }

    /// Shift the Rect along the y axis.
    pub fn shift_y(self, y: S) -> Self {
        Rect {
            y: self.y.shift(y),
            ..self
        }
    }

    /// Shift the Rect by the given vector.
    pub fn shift(self, v: Vector2<S>) -> Self {
        self.shift_x(v.x).shift_y(v.y)
    }

    /// Does the given point touch the Rectangle.
    pub fn contains(&self, p: Point2<S>) -> bool {
        self.x.contains(p.x) && self.y.contains(p.y)
    }

    /// Stretches the closest edge(s) to the given point if the point lies outside of the Rect area.
    pub fn stretch_to_point(self, p: Point2<S>) -> Self {
        let Rect { x, y } = self;
        Rect {
            x: x.stretch_to_value(p.x),
            y: y.stretch_to_value(p.y),
        }
    }

    /// Align `self`'s right edge with the left edge of the `other` **Rect**.
    pub fn left_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_before(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s left edge with the right dge of the `other` **Rect**.
    pub fn right_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_after(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s top edge with the bottom edge of the `other` **Rect**.
    pub fn below(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_before(other.x),
        }
    }

    /// Align `self`'s bottom edge with the top edge of the `other` **Rect**.
    pub fn above(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_after(other.x),
        }
    }

    /// Align `self` to `other` along the *x* axis in accordance with the given `Align` variant.
    pub fn align_x_of(self, align: Align, other: Self) -> Self {
        Rect {
            x: self.x.align_to(align, other.x),
            y: self.y,
        }
    }

    /// Align `self` to `other` along the *y* axis in accordance with the given `Align` variant.
    pub fn align_y_of(self, align: Align, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_to(align, other.y),
        }
    }

    /// Align `self`'s left edge with the left edge of the `other` **Rect**.
    pub fn align_left_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_start_of(other.x),
            y: self.y,
        }
    }

    /// Align the middle of `self` with the middle of the `other` **Rect** along the *x* axis.
    pub fn align_middle_x_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_middle_of(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s right edge with the right edge of the `other` **Rect**.
    pub fn align_right_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_end_of(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s bottom edge with the bottom edge of the `other` **Rect**.
    pub fn align_bottom_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_start_of(other.y),
        }
    }

    /// Align the middle of `self` with the middle of the `other` **Rect** along the *y* axis.
    pub fn align_middle_y_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_middle_of(other.y),
        }
    }

    /// Align `self`'s top edge with the top edge of the `other` **Rect**.
    pub fn align_top_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_end_of(other.y),
        }
    }

    /// Place `self` along the top left edges of the `other` **Rect**.
    pub fn top_left_of(self, other: Self) -> Self {
        self.align_left_of(other).align_top_of(other)
    }

    /// Place `self` along the top right edges of the `other` **Rect**.
    pub fn top_right_of(self, other: Self) -> Self {
        self.align_right_of(other).align_top_of(other)
    }

    /// Place `self` along the bottom left edges of the `other` **Rect**.
    pub fn bottom_left_of(self, other: Self) -> Self {
        self.align_left_of(other).align_bottom_of(other)
    }

    /// Place `self` along the bottom right edges of the `other` **Rect**.
    pub fn bottom_right_of(self, other: Self) -> Self {
        self.align_right_of(other).align_bottom_of(other)
    }

    /// Place `self` in the middle of the top edge of the `other` **Rect**.
    pub fn mid_top_of(self, other: Self) -> Self {
        self.align_middle_x_of(other).align_top_of(other)
    }

    /// Place `self` in the middle of the bottom edge of the `other` **Rect**.
    pub fn mid_bottom_of(self, other: Self) -> Self {
        self.align_middle_x_of(other).align_bottom_of(other)
    }

    /// Place `self` in the middle of the left edge of the `other` **Rect**.
    pub fn mid_left_of(self, other: Self) -> Self {
        self.align_left_of(other).align_middle_y_of(other)
    }

    /// Place `self` in the middle of the right edge of the `other` **Rect**.
    pub fn mid_right_of(self, other: Self) -> Self {
        self.align_right_of(other).align_middle_y_of(other)
    }

    /// Place `self` directly in the middle of the `other` **Rect**.
    pub fn middle_of(self, other: Self) -> Self {
        self.align_middle_x_of(other).align_middle_y_of(other)
    }

    /// Return the **Corner** of `self` that is closest to the given **Point**.
    pub fn closest_corner(&self, p: Point2<S>) -> Corner {
        let x_edge = self.x.closest_edge(p.x);
        let y_edge = self.y.closest_edge(p.y);
        match (x_edge, y_edge) {
            (Edge::Start, Edge::Start) => Corner::BottomLeft,
            (Edge::Start, Edge::End) => Corner::TopLeft,
            (Edge::End, Edge::Start) => Corner::BottomRight,
            (Edge::End, Edge::End) => Corner::TopRight,
        }
    }

    /// Thee four corners of the `Rect`.
    pub fn corners(&self) -> Quad<Point2<S>> {
        let (l, r, b, t) = self.l_r_b_t();
        let lb = [l, b].into();
        let lt = [l, t].into();
        let rt = [r, t].into();
        let rb = [r, b].into();
        Quad::from([lb, lt, rt, rb])
    }

    /// An iterator yielding the four corners of the `Rect`.
    pub fn corners_iter(&self) -> Corners<S> {
        let rect = *self;
        let index = 0;
        Corners { rect, index }
    }

    /// Return two `Tri`s that represent the `Rect`.
    pub fn triangles(&self) -> (Tri<Point2<S>>, Tri<Point2<S>>) {
        self.corners().triangles()
    }

    /// An iterator yielding the `Rect`'s two `Tri`'s.
    pub fn triangles_iter(self) -> Triangles<S> {
        self.corners().triangles_iter()
    }

    /// The four ranges used for the `Rect`'s four subdivisions.
    pub fn subdivision_ranges(&self) -> SubdivisionRanges<S> {
        let (x, y) = self.x_y();
        let x_a = Range::new(self.x.start, x);
        let x_b = Range::new(x, self.x.end);
        let y_a = Range::new(self.y.start, y);
        let y_b = Range::new(y, self.y.end);
        SubdivisionRanges { x_a, x_b, y_a, y_b }
    }

    /// Divide the `Rect` in half along the *x* and *y* axes and return the four subdivisions.
    ///
    /// Subdivisions are yielded in the following order:
    ///
    /// 1. Bottom left
    /// 2. Bottom right
    /// 3. Top left
    /// 4. Top right
    pub fn subdivisions(&self) -> [Self; NUM_SUBDIVISIONS as usize] {
        self.subdivision_ranges().rects()
    }

    /// The same as `subdivisions` but each subdivision is yielded via the returned `Iterator`.
    pub fn subdivisions_iter(&self) -> Subdivisions<S> {
        self.subdivision_ranges().rects_iter()
    }

    /// Produce the corner at the given index.
    pub fn corner_at_index(&self, index: u8) -> Option<Point2<S>> {
        match index {
            0 => Some(corner_from_index!(self, 0)),
            1 => Some(corner_from_index!(self, 1)),
            2 => Some(corner_from_index!(self, 2)),
            3 => Some(corner_from_index!(self, 3)),
            _ => None,
        }
    }
}

impl<S> SubdivisionRanges<S>
where
    S: Copy,
{
    /// The `Rect`s representing each of the four subdivisions.
    ///
    /// Subdivisions are yielded in the following order:
    ///
    /// 1. Bottom left
    /// 2. Bottom right
    /// 3. Top left
    /// 4. Top right
    pub fn rects(&self) -> [Rect<S>; NUM_SUBDIVISIONS as usize] {
        let r1 = subdivision_from_index!(self, 0);
        let r2 = subdivision_from_index!(self, 1);
        let r3 = subdivision_from_index!(self, 2);
        let r4 = subdivision_from_index!(self, 3);
        [r1, r2, r3, r4]
    }

    /// The same as `rects` but each subdivision is yielded via the returned `Iterator`.
    pub fn rects_iter(self) -> Subdivisions<S> {
        Subdivisions {
            ranges: self,
            subdivision_index: 0,
        }
    }

    // The subdivision at the given index within the range 0..NUM_SUBDIVISIONS.
    fn subdivision_at_index(&self, index: u8) -> Option<Rect<S>> {
        let rect = match index {
            0 => subdivision_from_index!(self, 0),
            1 => subdivision_from_index!(self, 1),
            2 => subdivision_from_index!(self, 2),
            3 => subdivision_from_index!(self, 3),
            _ => return None,
        };
        Some(rect)
    }
}

impl<S> Rect<S>
where
    S: BaseNum + Neg<Output = S>,
{
    /// The width of the Rect.
    pub fn w(&self) -> S {
        self.x.len()
    }

    /// The height of the Rect.
    pub fn h(&self) -> S {
        self.y.len()
    }

    /// The total dimensions of the Rect.
    pub fn wh(&self) -> Vector2<S> {
        [self.w(), self.h()].into()
    }

    /// The width and height of the Rect as a tuple.
    pub fn w_h(&self) -> (S, S) {
        (self.w(), self.h())
    }

    /// Convert the Rect to a `Point` and `Dimensions`.
    pub fn xy_wh(&self) -> (Point2<S>, Vector2<S>) {
        (self.xy(), self.wh())
    }

    /// The Rect's centered coordinates and dimensions in a tuple.
    pub fn x_y_w_h(&self) -> (S, S, S, S) {
        let (xy, wh) = self.xy_wh();
        (xy[0], xy[1], wh[0], wh[1])
    }

    /// The length of the longest side of the rectangle.
    pub fn len(&self) -> S {
        math::partial_max(self.w(), self.h())
    }

    /// The left and top edges of the **Rect** along with the width and height.
    pub fn l_t_w_h(&self) -> (S, S, S, S) {
        let (w, h) = self.w_h();
        (self.left(), self.top(), w, h)
    }

    /// The left and bottom edges of the **Rect** along with the width and height.
    pub fn l_b_w_h(&self) -> (S, S, S, S) {
        let (w, h) = self.w_h();
        (self.left(), self.bottom(), w, h)
    }

    /// The Rect with some padding applied to the left edge.
    pub fn pad_left(self, pad: S) -> Self {
        Rect {
            x: self.x.pad_start(pad),
            ..self
        }
    }

    /// The Rect with some padding applied to the right edge.
    pub fn pad_right(self, pad: S) -> Self {
        Rect {
            x: self.x.pad_end(pad),
            ..self
        }
    }

    /// The rect with some padding applied to the bottom edge.
    pub fn pad_bottom(self, pad: S) -> Self {
        Rect {
            y: self.y.pad_start(pad),
            ..self
        }
    }

    /// The Rect with some padding applied to the top edge.
    pub fn pad_top(self, pad: S) -> Self {
        Rect {
            y: self.y.pad_end(pad),
            ..self
        }
    }

    /// The Rect with some padding amount applied to each edge.
    pub fn pad(self, pad: S) -> Self {
        let Rect { x, y } = self;
        Rect {
            x: x.pad(pad),
            y: y.pad(pad),
        }
    }

    /// The Rect with some padding applied.
    pub fn padding(self, padding: Padding<S>) -> Self {
        Rect {
            x: self.x.pad_ends(padding.x.start, padding.x.end),
            y: self.y.pad_ends(padding.y.start, padding.y.end),
        }
    }

    /// Returns a `Rect` with a position relative to the given position on the *x* axis.
    pub fn relative_to_x(self, x: S) -> Self {
        Rect {
            x: self.x.shift(-x),
            ..self
        }
    }

    /// Returns a `Rect` with a position relative to the given position on the *y* axis.
    pub fn relative_to_y(self, y: S) -> Self {
        Rect {
            y: self.y.shift(-y),
            ..self
        }
    }

    /// Returns a `Rect` with a position relative to the given position.
    pub fn relative_to(self, p: Point2<S>) -> Self {
        self.relative_to_x(p.x).relative_to_y(p.y)
    }
}

impl<S> Iterator for Subdivisions<S>
where
    S: Copy,
{
    type Item = Rect<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sd) = self.ranges.subdivision_at_index(self.subdivision_index) {
            self.subdivision_index += 1;
            return Some(sd);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> DoubleEndedIterator for Subdivisions<S>
where
    S: Copy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_index = self.subdivision_index + 1;
        if let Some(sd) = self
            .ranges
            .subdivision_at_index(NUM_SUBDIVISIONS - next_index)
        {
            self.subdivision_index = next_index;
            return Some(sd);
        }
        None
    }
}

impl<S> ExactSizeIterator for Subdivisions<S>
where
    S: Copy,
{
    fn len(&self) -> usize {
        NUM_SUBDIVISIONS as usize - self.subdivision_index as usize
    }
}

impl<S> Iterator for Corners<S>
where
    S: BaseNum,
{
    type Item = Point2<S>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(corner) = self.rect.corner_at_index(self.index) {
            self.index += 1;
            return Some(corner);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> DoubleEndedIterator for Corners<S>
where
    S: BaseNum,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_index = self.index + 1;
        if let Some(corner) = self.rect.corner_at_index(NUM_CORNERS - next_index) {
            self.index = next_index;
            return Some(corner);
        }
        None
    }
}

impl<S> ExactSizeIterator for Corners<S>
where
    S: BaseNum,
{
    fn len(&self) -> usize {
        (NUM_CORNERS - self.index) as usize
    }
}
