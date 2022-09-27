//! Display an interactive rotating knob that controls a [`NormalParam`]
//!
//! [`NormalParam`]: ../core/normal_param/struct.NormalParam.html

use std::fmt::Debug;

use iced_native::widget::tree::{self, Tree};
use iced_native::{
    event, keyboard, layout, mouse, touch, Clipboard, Element, Event, Layout,
    Length, Point, Rectangle, Shell, Size, Widget,
};

use crate::core::{ModulationRange, Normal, NormalParam};
use crate::native::{text_marks, tick_marks};

static DEFAULT_SIZE: u16 = 30;
static DEFAULT_SCALAR: f32 = 0.00385;
static DEFAULT_WHEEL_SCALAR: f32 = 0.01;
static DEFAULT_MODIFIER_SCALAR: f32 = 0.02;

/// A rotating knob GUI widget that controls a [`NormalParam`]
///
/// [`NormalParam`]: ../../core/normal_param/struct.NormalParam.html
#[allow(missing_debug_implementations)]
pub struct Knob<'a, Message, Renderer: self::Renderer> {
    normal_param: NormalParam,
    size: Length,
    on_change: Box<dyn Fn(Normal) -> Message + 'a>,
    on_drag_start: Box<dyn Fn() -> Option<Message> + 'a>,
    on_drag_end: Box<dyn Fn() -> Option<Message> + 'a>,
    scalar: f32,
    wheel_scalar: f32,
    modifier_scalar: f32,
    modifier_keys: keyboard::Modifiers,
    bipolar_center: Option<Normal>,
    style: Renderer::Style,
    tick_marks: Option<&'a tick_marks::Group>,
    text_marks: Option<&'a text_marks::Group>,
    mod_range_1: Option<&'a ModulationRange>,
    mod_range_2: Option<&'a ModulationRange>,
}

impl<'a, Message, Renderer> Knob<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer,
{
    /// Creates a new [`Knob`].
    ///
    /// It expects:
    ///   * the [`NormalParam`] of the [`Knob`]
    ///   * a function that will be called when the [`Knob`] is turned.
    ///
    /// [`NormalParam`]: struct.NormalParam.html
    /// [`Knob`]: struct.Knob.html
    pub fn new<F, G, H>(
        normal_param: NormalParam,
        on_change: F,
        // FIXME those two should probably be optionals
        on_drag_start: G,
        on_drag_end: H,
    ) -> Self
    where
        F: 'a + Fn(Normal) -> Message,
        G: 'a + Fn() -> Option<Message>,
        H: 'a + Fn() -> Option<Message>,
    {
        Knob {
            normal_param,
            size: Length::from(Length::Units(DEFAULT_SIZE)),
            on_change: Box::new(on_change),
            on_drag_start: Box::new(on_drag_start),
            on_drag_end: Box::new(on_drag_end),
            scalar: DEFAULT_SCALAR,
            wheel_scalar: DEFAULT_WHEEL_SCALAR,
            modifier_scalar: DEFAULT_MODIFIER_SCALAR,
            modifier_keys: keyboard::Modifiers::CTRL,
            bipolar_center: None,
            style: Renderer::Style::default(),
            tick_marks: None,
            text_marks: None,
            mod_range_1: None,
            mod_range_2: None,
        }
    }

    /// Sets the diameter of the [`Knob`]. The default size is
    /// `Length::from(Length::Units(31))`.
    ///
    /// [`Knob`]: struct.Knob.html
    pub fn size(mut self, size: Length) -> Self {
        self.size = size;
        self
    }

    /// Sets the style of the [`Knob`].
    ///
    /// [`Knob`]: struct.Knob.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets how much the [`Normal`] value will change for the [`Knob`] per `y`
    /// pixel movement of the mouse.
    ///
    /// The default value is `0.00385`
    ///
    /// [`Knob`]: struct.Knob.html
    /// [`Normal`]: ../../core/struct.Normal.html
    pub fn scalar(mut self, scalar: f32) -> Self {
        self.scalar = scalar;
        self
    }

    /// Sets how much the [`Normal`] value will change for the [`Knob`] per line scrolled
    /// by the mouse wheel.
    ///
    /// This can be set to `0.0` to disable the scroll wheel from moving the parameter.
    ///
    /// The default value is `0.01`
    ///
    /// [`Knob`]: struct.Knob.html
    /// [`Normal`]: ../../core/struct.Normal.html
    pub fn wheel_scalar(mut self, wheel_scalar: f32) -> Self {
        self.wheel_scalar = wheel_scalar;
        self
    }

    /// Sets the modifier keys of the [`Knob`].
    ///
    /// The default modifier key is `Ctrl`.
    ///
    /// [`Knob`]: struct.Knob.html
    pub fn modifier_keys(mut self, modifier_keys: keyboard::Modifiers) -> Self {
        self.modifier_keys = modifier_keys;
        self
    }

    /// Sets the scalar to use when the user drags the knobs while holding down
    /// the modifier key. This is multiplied to the value set by
    /// `Knob::scalar()` (which the default is `0.00385`).
    ///
    /// For example, a `modifier_scalar` of `0.5` will cause the knob to turn
    /// half as fast when the modifier key is down.
    ///
    /// The default `modifier_scalar` is `0.02`, and the default modifier key
    /// is `Ctrl`.
    ///
    /// [`Knob`]: struct.Knob.html
    pub fn modifier_scalar(mut self, scalar: f32) -> Self {
        self.modifier_scalar = scalar;
        self
    }

    /// Sets the tick marks to display. Note your [`StyleSheet`] must
    /// also implement `tick_marks_style(&self) -> Option<tick_marks::Style>` for
    /// them to display (which the default style does).
    ///
    /// [`StyleSheet`]: ../../style/knob/trait.StyleSheet.html
    pub fn tick_marks(mut self, tick_marks: &'a tick_marks::Group) -> Self {
        self.tick_marks = Some(tick_marks);
        self
    }

    /// Sets the text marks to display. Note your [`StyleSheet`] must
    /// also implement `text_marks_style(&self) -> Option<text_marks::Style>` for
    /// them to display (which the default style does).
    ///
    /// [`StyleSheet`]: ../../style/knob/trait.StyleSheet.html
    pub fn text_marks(mut self, text_marks: &'a text_marks::Group) -> Self {
        self.text_marks = Some(text_marks);
        self
    }

    /// Sets a [`ModulationRange`] to display. Note your [`StyleSheet`] must
    /// also implement `mod_range_style(&self) -> Option<ModRangeStyle>` for
    /// them to display.
    ///
    /// [`ModulationRange`]: ../../core/struct.ModulationRange.html
    /// [`StyleSheet`]: ../../style/v_slider/trait.StyleSheet.html
    pub fn mod_range(mut self, mod_range: &'a ModulationRange) -> Self {
        self.mod_range_1 = Some(mod_range);
        self
    }

    /// Sets a second [`ModulationRange`] to display. Note your [`StyleSheet`] must
    /// also implement `mod_range_style_2(&self) -> Option<ModRangeStyle>` for
    /// them to display.
    ///
    /// [`ModulationRange`]: ../../core/struct.ModulationRange.html
    /// [`StyleSheet`]: ../../style/v_slider/trait.StyleSheet.html
    pub fn mod_range_2(mut self, mod_range: &'a ModulationRange) -> Self {
        self.mod_range_1 = Some(mod_range);
        self
    }

    /// Sets the value to be considered the center of the [`Knob`]. Only has
    /// an effect when using [`ArcBipolarStyle`].
    ///
    /// [`Knob`]: struct.Knob.html
    /// [`ArcBipolarStyle`]: ../../style/knob/struct.ArcBipolarStyle.html
    pub fn bipolar_center(mut self, bipolar_center: Normal) -> Self {
        self.bipolar_center = Some(bipolar_center);
        self
    }

    fn move_virtual_slider(
        &mut self,
        state: &mut State,
        shell: &mut Shell<'_, Message>,
        mut normal_delta: f32,
    ) {
        if state.pressed_modifiers.contains(self.modifier_keys) {
            normal_delta *= self.modifier_scalar;
        }

        self.normal_param.value =
            Normal::new(state.continuous_normal - normal_delta);
        state.continuous_normal = self.normal_param.value.as_f32();

        shell.publish((self.on_change)(self.normal_param.value));
    }
}

/// The local state of a [`Knob`].
///
/// [`Knob`]: struct.Knob.html
#[derive(Debug, Clone)]
struct State {
    is_dragging: bool,
    prev_drag_y: f32,
    continuous_normal: f32,
    pressed_modifiers: keyboard::Modifiers,
    last_click: Option<mouse::Click>,
    tick_marks_cache: crate::graphics::tick_marks::PrimitiveCache,
    text_marks_cache: crate::graphics::text_marks::PrimitiveCache,
}

impl State {
    /// Creates a new [`Knob`] state.
    ///
    /// It expects:
    /// * current [`Normal`] value for the [`Knob`]
    ///
    /// [`Normal`]: ../../core/normal/struct.Normal.html
    /// [`Knob`]: struct.Knob.html
    fn new(normal: Normal) -> Self {
        Self {
            is_dragging: false,
            prev_drag_y: 0.0,
            continuous_normal: normal.as_f32(),
            pressed_modifiers: Default::default(),
            last_click: None,
            tick_marks_cache: Default::default(),
            text_marks_cache: Default::default(),
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Knob<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.normal_param.value))
    }

    fn width(&self) -> Length {
        self.size
    }

    fn height(&self) -> Length {
        self.size
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.size).height(self.size);

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let state = state.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if state.is_dragging {
                    let normal_delta =
                        (cursor_position.y - state.prev_drag_y) * self.scalar;

                    state.prev_drag_y = cursor_position.y;

                    self.move_virtual_slider(state, shell, normal_delta);

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if self.wheel_scalar == 0.0 {
                    return event::Status::Ignored;
                }

                if layout.bounds().contains(cursor_position) {
                    let lines = match delta {
                        iced_native::mouse::ScrollDelta::Lines {
                            y, ..
                        } => y,
                        iced_native::mouse::ScrollDelta::Pixels {
                            y, ..
                        } => {
                            if y > 0.0 {
                                1.0
                            } else if y < 0.0 {
                                -1.0
                            } else {
                                0.0
                            }
                        }
                    };

                    if lines != 0.0 {
                        let normal_delta = -lines * self.wheel_scalar;

                        self.move_virtual_slider(state, shell, normal_delta);

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    let click =
                        mouse::Click::new(cursor_position, state.last_click);

                    match click.kind() {
                        mouse::click::Kind::Single => {
                            state.is_dragging = true;
                            state.prev_drag_y = cursor_position.y;
                            state.continuous_normal =
                                self.normal_param.value.as_f32();

                            if let Some(message) = (self.on_drag_start)() {
                                shell.publish(message);
                            }
                        }
                        _ => {
                            state.is_dragging = false;

                            self.normal_param.value = self.normal_param.default;
                            state.continuous_normal =
                                self.normal_param.default.as_f32();

                            shell.publish((self.on_change)(
                                self.normal_param.value,
                            ));
                        }
                    }

                    state.last_click = Some(click);

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    state.continuous_normal = self.normal_param.value.as_f32();

                    if let Some(message) = (self.on_drag_end)() {
                        shell.publish(message);
                    }

                    return event::Status::Captured;
                }
            }
            Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed { modifiers, .. } => {
                    state.pressed_modifiers = modifiers;

                    return event::Status::Captured;
                }
                keyboard::Event::KeyReleased { modifiers, .. } => {
                    state.pressed_modifiers = modifiers;

                    return event::Status::Captured;
                }
                keyboard::Event::ModifiersChanged(modifiers) => {
                    state.pressed_modifiers = modifiers;

                    return event::Status::Captured;
                }
                _ => {}
            },
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        _theme: &Renderer::Theme,
        _style: &iced_native::renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let state = state.state.downcast_ref::<State>();
        renderer.draw(
            layout.bounds(),
            cursor_position,
            self.normal_param.value,
            self.bipolar_center,
            state.is_dragging,
            self.mod_range_1,
            self.mod_range_2,
            self.tick_marks,
            self.text_marks,
            &self.style,
            &state.tick_marks_cache,
            &state.text_marks_cache,
        )
    }
}

/// The renderer of a [`Knob`].
///
/// Your renderer will need to implement this trait before being
/// able to use a [`Knob`] in your user interface.
///
/// [`Knob`]: struct.Knob.html
pub trait Renderer: iced_native::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// Draws a [`Knob`].
    ///
    /// It receives:
    ///   * the bounds of the [`Knob`]
    ///   * the current cursor position
    ///   * the current normal of the [`Knob`]
    ///   * optionally, a custom bipolar center value
    ///   * whether the knob is currently being dragged
    ///   * any tick marks to display
    ///   * any text marks to display
    ///   * the style of the [`Knob`]
    ///
    /// [`Knob`]: struct.Knob.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        normal: Normal,
        bipolar_center: Option<Normal>,
        is_dragging: bool,
        mod_range_1: Option<&ModulationRange>,
        mod_range_2: Option<&ModulationRange>,
        tick_marks: Option<&tick_marks::Group>,
        text_marks: Option<&text_marks::Group>,
        style: &Self::Style,
        tick_marks_cache: &crate::tick_marks::PrimitiveCache,
        text_marks_cache: &crate::text_marks::PrimitiveCache,
    );
}

impl<'a, Message, Renderer> From<Knob<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + self::Renderer,
{
    fn from(
        knob: Knob<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(knob)
    }
}
