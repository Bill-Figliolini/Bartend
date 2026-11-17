//! [`FramedCard`]: a container drawn with an inner line if the
//! [`Styling`]'s card frame has one, and optionally marks in its corners
//! (brackets or studs).
//!
//! The container draws its own fill and the outer line. The wrapper draws
//! the inner line and the corner marks over it, at the container's final
//! bounds. Doing it at draw time keeps the inner line flush with the outer
//! one when a tile is stretched to match its neighbours; a second,
//! nested container would keep its own height and fall short.

use std::fmt::{self, Debug};

use iced::{
    Background, Border, Color, Element, Event, Length, Padding, Rectangle, Renderer, Size, Theme,
    Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, overlay,
        renderer::{self, Quad, Renderer as _},
        widget::{Operation, Tree},
    },
    widget::Container,
};

use crate::presentation::styling::{CardFrame, Styling, TileCorners};

/// Gap between the outer line and the inner one.
const GAP: f32 = 2.0;
/// Width of the inner line.
const LINE: f32 = 1.0;
/// Length of each arm of a corner bracket.
const BRACKET_ARM: f32 = 10.0;
/// Thickness of a corner bracket.
const BRACKET_GIRTH: f32 = 2.0;
/// Width and height of a corner stud.
const STUD: f32 = 5.0;

/// Extra padding a card's content needs to clear a
/// [double](CardFrame::Double) frame's inner line. Kept under every
/// styling, so cards keep one size whatever their frame: a
/// [heavy](CardFrame::Heavy) frame's inner line sits a pixel further in,
/// within the card's usual padding.
pub const FRAME_PADDING: f32 = 4.0;

/// Picks the color of a card's inner line, and of its corner marks, from
/// the theme.
type Ink<'a> = Box<dyn Fn(&Theme) -> Color + 'a>;

/// A [`Container`] drawn as a framed card; see the
/// [module docs](self). Set its size with [`width`](Self::width) and
/// [`height`](Self::height) as on the container.
pub struct FramedCard<'a, M> {
    container: Container<'a, M>,
    /// Corner radius of the container; the inner line follows it.
    radius: f32,
    frame: CardFrame,
    line: Ink<'a>,
    corners: Option<(TileCorners, Ink<'a>)>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<'a, M: 'a> FramedCard<'a, M> {
    /// `container` styled with its fill and outer line, and padded to
    /// clear the inner line (see [`FRAME_PADDING`]); the inner line is
    /// drawn in the color `line` picks, with corners to match the
    /// `styling`'s, if the styling's cards have one.
    pub fn new(
        styling: &Styling,
        container: Container<'a, M>,
        line: impl Fn(&Theme) -> Color + 'a,
    ) -> Self {
        Self {
            container,
            radius: styling.radius,
            frame: styling.card_frame,
            line: Box::new(line),
            corners: None,
        }
    }

    /// Marks the corners as `corners` says, in the color `ink` picks.
    #[must_use]
    pub fn corners(mut self, corners: TileCorners, ink: impl Fn(&Theme) -> Color + 'a) -> Self {
        self.corners = Some((corners, Box::new(ink)));
        self
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.container = self.container.width(width);
        self
    }

    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.container = self.container.height(height);
        self
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<M> Debug for FramedCard<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramedCard")
            .field(
                "corners",
                &self.corners.as_ref().map(|(corners, _)| corners),
            )
            .finish_non_exhaustive()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<'a, M: 'a> From<FramedCard<'a, M>> for Element<'a, M> {
    fn from(card: FramedCard<'a, M>) -> Self {
        Element::new(Framed {
            content: card.container.into(),
            radius: card.radius,
            frame: card.frame,
            line: card.line,
            corners: card.corners,
        })
    }
}

/// The widget a [`FramedCard`] becomes: its container, with the inner line
/// and corner marks drawn on top.
struct Framed<'a, M> {
    content: Element<'a, M>,
    radius: f32,
    frame: CardFrame,
    line: Ink<'a>,
    corners: Option<(TileCorners, Ink<'a>)>,
}

/// How far in from the card's edge its inner line, and its corner marks,
/// run under `frame`: a [gap](GAP) inside the outer line.
fn inset_by(frame: CardFrame) -> f32 {
    frame.outline() + GAP
}

/// The rectangle a few pixels inside a card with `bounds` edged as `frame`
/// says, which the inner line and corner marks run along.
fn inset(bounds: Rectangle, frame: CardFrame) -> Rectangle {
    bounds.shrink(Padding::new(inset_by(frame)))
}

/// Where the inner line runs, for a card with `bounds` edged as `frame`
/// says, or `None` if it has only its outer line.
fn inner_line(bounds: Rectangle, frame: CardFrame) -> Option<Rectangle> {
    match frame {
        CardFrame::Single => None,
        CardFrame::Double | CardFrame::Heavy => Some(inset(bounds, frame)),
    }
}

/// The corner radius of the inner line of a card whose corners have
/// `radius` and which is edged as `frame` says: smaller by the inset, so
/// the two lines stay evenly apart.
fn inner_radius(radius: f32, frame: CardFrame) -> f32 {
    (radius - inset_by(frame)).max(0.0)
}

/// The rectangles to fill to mark the corners of a card with `bounds`,
/// edged as `frame` says, as `corners` says.
fn corner_marks(bounds: Rectangle, frame: CardFrame, corners: TileCorners) -> Vec<Rectangle> {
    let line = inset(bounds, frame);
    match corners {
        TileCorners::None => Vec::new(),
        TileCorners::Brackets => bracket_bars(line).to_vec(),
        TileCorners::Studs => studs(line).to_vec(),
    }
}

/// The eight bars making up the corner brackets inside `line`, where a
/// card's inner line runs: for each corner, one across and one down.
fn bracket_bars(line: Rectangle) -> [Rectangle; 8] {
    let across = Size::new(BRACKET_ARM, BRACKET_GIRTH);
    let down = Size::new(BRACKET_GIRTH, BRACKET_ARM);
    let left = line.x;
    let top = line.y;
    let right = line.x + line.width;
    let bottom = line.y + line.height;
    let bar = |x: f32, y: f32, size: Size| Rectangle::new([x, y].into(), size);
    [
        bar(left, top, across),
        bar(left, top, down),
        bar(right - BRACKET_ARM, top, across),
        bar(right - BRACKET_GIRTH, top, down),
        bar(left, bottom - BRACKET_GIRTH, across),
        bar(left, bottom - BRACKET_ARM, down),
        bar(right - BRACKET_ARM, bottom - BRACKET_GIRTH, across),
        bar(right - BRACKET_GIRTH, bottom - BRACKET_ARM, down),
    ]
}

/// The four squares making up the corner studs inside `line`, where a
/// card's inner line runs, one tucked into each corner.
fn studs(line: Rectangle) -> [Rectangle; 4] {
    let size = Size::new(STUD, STUD);
    let left = line.x;
    let top = line.y;
    let right = line.x + line.width - STUD;
    let bottom = line.y + line.height - STUD;
    [[left, top], [right, top], [left, bottom], [right, bottom]]
        .map(|corner| Rectangle::new(corner.into(), size))
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<M> Widget<M, Theme, Renderer> for Framed<'_, M> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        let bounds = layout.bounds();
        if let Some(line) = inner_line(bounds, self.frame) {
            renderer.fill_quad(
                Quad {
                    bounds: line,
                    border: Border {
                        width: LINE,
                        color: (self.line)(theme),
                        radius: inner_radius(self.radius, self.frame).into(),
                    },
                    snap: true,
                    ..Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }
        if let Some((corners, ink)) = &self.corners {
            let color = ink(theme);
            for mark in corner_marks(bounds, self.frame, *corners) {
                renderer.fill_quad(
                    Quad {
                        bounds: mark,
                        snap: true,
                        ..Quad::default()
                    },
                    color,
                );
            }
        }
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD: Rectangle = Rectangle {
        x: 100.0,
        y: 50.0,
        width: 200.0,
        height: 80.0,
    };

    const FRAMES: [CardFrame; 3] = [CardFrame::Single, CardFrame::Double, CardFrame::Heavy];

    #[test]
    fn the_inner_line_is_inset_on_every_side() {
        let inset = Rectangle {
            x: 103.0,
            y: 53.0,
            width: 194.0,
            height: 74.0,
        };
        assert_eq!(inner_line(CARD, CardFrame::Double), Some(inset));
    }

    #[test]
    fn a_heavy_frames_inner_line_keeps_the_same_gap_inside_its_thicker_outline() {
        let inset = Rectangle {
            x: 104.0,
            y: 54.0,
            width: 192.0,
            height: 72.0,
        };
        assert_eq!(inner_line(CARD, CardFrame::Heavy), Some(inset));
        for frame in FRAMES {
            assert!((inset_by(frame) - frame.outline() - GAP).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn the_frame_padding_clears_a_double_frames_inner_line() {
        assert!((FRAME_PADDING - (inset_by(CardFrame::Double) + LINE)).abs() < f32::EPSILON);
    }

    #[test]
    fn single_edged_cards_have_no_inner_line() {
        assert_eq!(inner_line(CARD, CardFrame::Single), None);
    }

    #[test]
    fn the_inner_lines_corners_follow_the_cards() {
        for frame in FRAMES {
            let inset = inset_by(frame);
            assert!(inner_radius(0.0, frame).abs() < f32::EPSILON);
            assert!(inner_radius(inset - 1.0, frame).abs() < f32::EPSILON);
            assert!((inner_radius(8.0, frame) - (8.0 - inset)).abs() < f32::EPSILON);
        }
    }

    /// Checks that every one of `marks` lies inside `line`, and that each
    /// of its corners is covered by `per_corner` of them.
    fn assert_marks_corners(marks: &[Rectangle], line: Rectangle, per_corner: usize) {
        for mark in marks {
            let inside = mark.x >= line.x
                && mark.y >= line.y
                && mark.x + mark.width <= line.x + line.width
                && mark.y + mark.height <= line.y + line.height;
            assert!(inside, "{mark:?}");
        }
        let corners = [
            [line.x, line.y],
            [line.x + line.width - 1.0, line.y],
            [line.x, line.y + line.height - 1.0],
            [line.x + line.width - 1.0, line.y + line.height - 1.0],
        ];
        for corner in corners {
            let covering = marks
                .iter()
                .filter(|mark| mark.contains(corner.into()))
                .count();
            assert_eq!(covering, per_corner, "{corner:?}");
        }
    }

    #[test]
    fn brackets_sit_inside_the_inset_at_its_corners() {
        for frame in FRAMES {
            let marks = corner_marks(CARD, frame, TileCorners::Brackets);
            assert_eq!(marks.len(), 8);
            assert_marks_corners(&marks, inset(CARD, frame), 2);
        }
    }

    #[test]
    fn studs_are_squares_tucked_into_the_insets_corners() {
        for frame in FRAMES {
            let marks = corner_marks(CARD, frame, TileCorners::Studs);
            assert_eq!(marks.len(), 4);
            for mark in &marks {
                assert_eq!(mark.size(), Size::new(STUD, STUD));
            }
            assert_marks_corners(&marks, inset(CARD, frame), 1);
        }
    }

    #[test]
    fn no_corner_marks_means_none() {
        for frame in FRAMES {
            assert_eq!(corner_marks(CARD, frame, TileCorners::None), Vec::new());
        }
    }
}
