//! [`DecoCard`]: a container drawn with an art deco double border, and
//! optionally champagne brackets in its corners.
//!
//! The container draws its own fill and the outer line. The wrapper draws
//! the inner line and the brackets over it, at the container's final
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

/// Gap from the card's edge to its inner line.
const INSET: f32 = 3.0;
/// Width of the inner line.
const LINE: f32 = 1.0;
/// Length of each arm of a corner bracket.
const BRACKET_ARM: f32 = 10.0;
/// Thickness of a corner bracket.
const BRACKET_GIRTH: f32 = 2.0;

/// Extra padding a card's content needs to clear its inner line.
pub const FRAME_PADDING: f32 = INSET + LINE;

/// Picks the colour of a card's inner line, and of its brackets, from the
/// theme.
type Ink<'a> = Box<dyn Fn(&Theme) -> Color + 'a>;

/// A [`Container`] drawn as an art deco card; see the
/// [module docs](self). Set its size with [`width`](Self::width) and
/// [`height`](Self::height) as on the container.
pub struct DecoCard<'a, M> {
    container: Container<'a, M>,
    line: Ink<'a>,
    brackets: Option<Ink<'a>>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<'a, M: 'a> DecoCard<'a, M> {
    /// `container` styled with its fill and outer line, and padded to
    /// clear the inner line (see [`FRAME_PADDING`]); the inner line is
    /// drawn in the colour `line` picks.
    pub fn new(container: Container<'a, M>, line: impl Fn(&Theme) -> Color + 'a) -> Self {
        Self {
            container,
            line: Box::new(line),
            brackets: None,
        }
    }

    /// Adds brackets in the corners, in the colour `ink` picks.
    #[must_use]
    pub fn brackets(mut self, ink: impl Fn(&Theme) -> Color + 'a) -> Self {
        self.brackets = Some(Box::new(ink));
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
impl<M> Debug for DecoCard<'_, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecoCard")
            .field("brackets", &self.brackets.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<'a, M: 'a> From<DecoCard<'a, M>> for Element<'a, M> {
    fn from(card: DecoCard<'a, M>) -> Self {
        Element::new(Framed {
            content: card.container.into(),
            line: card.line,
            brackets: card.brackets,
        })
    }
}

/// The widget a [`DecoCard`] becomes: its container, with the inner line
/// and brackets drawn on top.
struct Framed<'a, M> {
    content: Element<'a, M>,
    line: Ink<'a>,
    brackets: Option<Ink<'a>>,
}

/// Where the inner line runs, for a card with `bounds`.
fn inner_line(bounds: Rectangle) -> Rectangle {
    bounds.shrink(Padding::new(INSET))
}

/// The eight bars making up the corner brackets of a card with `bounds`:
/// for each corner, one across and one down, laid along its inner line.
fn bracket_bars(bounds: Rectangle) -> [Rectangle; 8] {
    let line = inner_line(bounds);
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
        renderer.fill_quad(
            Quad {
                bounds: inner_line(bounds),
                border: Border {
                    width: LINE,
                    color: (self.line)(theme),
                    radius: 0.0.into(),
                },
                snap: true,
                ..Quad::default()
            },
            Background::Color(Color::TRANSPARENT),
        );
        if let Some(ink) = &self.brackets {
            let color = ink(theme);
            for bar in bracket_bars(bounds) {
                renderer.fill_quad(
                    Quad {
                        bounds: bar,
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

    #[test]
    fn the_inner_line_is_inset_on_every_side() {
        assert_eq!(
            inner_line(CARD),
            Rectangle {
                x: 103.0,
                y: 53.0,
                width: 194.0,
                height: 74.0,
            }
        );
    }

    #[test]
    fn brackets_sit_inside_the_inner_line_at_its_corners() {
        let line = inner_line(CARD);
        let bars = bracket_bars(CARD);
        for bar in bars {
            let inside = bar.x >= line.x
                && bar.y >= line.y
                && bar.x + bar.width <= line.x + line.width
                && bar.y + bar.height <= line.y + line.height;
            assert!(inside, "{bar:?}");
        }
        // Each corner of the inner line is covered by two bars.
        let corners = [
            [line.x, line.y],
            [line.x + line.width - 1.0, line.y],
            [line.x, line.y + line.height - 1.0],
            [line.x + line.width - 1.0, line.y + line.height - 1.0],
        ];
        for corner in corners {
            let covering = bars
                .iter()
                .filter(|bar| bar.contains(corner.into()))
                .count();
            assert_eq!(covering, 2, "{corner:?}");
        }
    }
}
