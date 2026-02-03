//! //TODO: content is cloned to render. wasteful.

use core::f32;
use std::cell::RefCell;

use iced::advanced::graphics::core::keyboard;
use iced::advanced::layout::{self, Layout};
use iced::advanced::text::{self, Editor};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, renderer, Clipboard, Shell};
use iced::mouse::ScrollDelta;
use iced::widget::text::LineHeight;
use iced::Event;
use iced::{alignment, window, Pixels};
use iced::{border, mouse};
use iced::{color, Padding};
use iced::{Color, Element, Length, Rectangle, Size};

use advanced::text::editor;

pub use text::editor::Action;

pub struct Paper<'a, R, Message>
where
    R: text::Renderer,
{
    padding: Padding,
    handle: &'a PaperHandle<R>,
    color_background: Color,
    color_fill: Color,
    on_edit: Option<Box<dyn Fn(editor::Action) -> Message + 'a>>,
}

pub struct PaperHandle<R = iced::Renderer>
where
    R: text::Renderer,
{
    core_editor: RefCell<R::Editor>,
}

impl<R> PaperHandle<R>
where
    R: text::Renderer,
{
    /// Creates an empty [`Content`].
    pub fn new(txt: &str) -> Self {
        let me = PaperHandle {
            core_editor: RefCell::new(R::Editor::with_text(txt)),
        };
        me
    }

    pub fn perform(&mut self, action: Action) {
        let internal = self.core_editor.get_mut();
        internal.perform(action);
    }
}

impl<'a, R, Message> Paper<'a, R, Message>
where
    R: text::Renderer,
{
    pub fn new(handle_arg: &'a PaperHandle<R>) -> Self {
        Self {
            padding: Padding {
                top: 50.,
                right: 5.0,
                bottom: 5.0,
                left: 5.0,
            },
            handle: handle_arg,
            color_background: Color::BLACK,
            color_fill: Color::WHITE,
            on_edit: None,
        }
    }

    pub fn set_style(&mut self, bg: Color, fill: Color) {
        self.color_background = bg;
        self.color_fill = fill;
    }

    pub fn on_action(mut self, on_edit: impl Fn(editor::Action) -> Message + 'a) -> Self {
        self.on_edit = Some(Box::new(on_edit));
        self
    }

    pub fn debug_line(&self, renderer: &mut R, bounds: Rectangle, content: String, line_id: u8) {
        let bounds = bounds.shrink(Padding::default().top((12 * line_id) as f32));

        // Debugging text here
        renderer.fill_text(
            advanced::Text {
                content: content,
                bounds: bounds.size(),
                size: Pixels(12.),
                line_height: LineHeight::Relative(1.3),
                font: renderer.default_font(),
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::WordOrGlyph,
            },
            bounds.position(),
            self.color_fill,
            bounds,
        );
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Paper<'_, Renderer, Message>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State {
            partial_scroll: 0.,
            scroll_lines: 0.,
        })
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        //let state = tree.state.downcast_mut::<State>();
        let mut core_editor = self.handle.core_editor.borrow_mut();

        //let limits = limits.width(400.);
        //println!("limits: {:?}", limits.max().width);

        core_editor.update(
            limits.shrink(self.padding).max(),
            //editor_bounds,
            renderer.default_font(),
            iced::Pixels(16.),
            LineHeight::Relative(1.3),
            widget::text::Wrapping::WordOrGlyph,
            &mut advanced::text::highlighter::PlainText,
        );

        layout::Node::new(limits.max())
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let Some(on_edit) = self.on_edit.as_ref() else {
            return;
        };
        let state = tree.state.downcast_mut::<State>();
        let _is_redraw = matches!(event, Event::Window(window::Event::RedrawRequested(_now)),);

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if cursor.is_over(layout.bounds()) =>
            {
                match delta {
                    ScrollDelta::Lines { x: _, y } => {
                        let mut lines = if y.abs() > 0.0 {
                            y.signum() * -(y.abs() * 4.0).max(1.0)
                        } else {
                            0.0
                        };

                        let bounds = layout.bounds();

                        if bounds.height >= i32::MAX as f32 {
                            return;
                        }

                        lines = lines + state.partial_scroll;
                        state.partial_scroll = lines.fract();
                        state.scroll_lines = lines;

                        shell.publish(on_edit(editor::Action::Scroll {
                            lines: lines as i32,
                        }));
                        shell.capture_event();
                    }
                    _ => {}
                }
            }

            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if key.as_ref() == keyboard::Key::Character("x") {
                    shell.publish(on_edit(editor::Action::Edit(editor::Edit::Insert('X'))));
                    shell.capture_event();
                }
            }

            _ => {}
        }
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let core_editor = self.handle.core_editor.borrow_mut();

        renderer.fill_quad(
            renderer::Quad {
                bounds: bounds,
                border: border::rounded(0),
                ..renderer::Quad::default()
            },
            self.color_background,
        );

        let bounds_editor = bounds.shrink(self.padding);

        // Debugging text here
        self.debug_line(renderer, bounds, format!("draw/bounds: {:?}", bounds), 0);

        self.debug_line(
            renderer,
            bounds,
            format!("draw/min_bounds: {:?}", core_editor.min_bounds()),
            1,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: bounds_editor,
                border: border::rounded(0),
                ..renderer::Quad::default()
            },
            color!(0x7a3c44),
        );

        renderer.fill_editor(
            &core_editor,
            bounds_editor.position(),
            self.color_fill,
            bounds_editor,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Paper<'a, Renderer, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
    Message: 'a,
{
    fn from(paper: Paper<'a, Renderer, Message>) -> Self {
        Self::new(paper)
    }
}

/// The state of a [`Paper`].
#[derive(Debug)]
pub struct State {
    partial_scroll: f32,
    scroll_lines: f32,
}
