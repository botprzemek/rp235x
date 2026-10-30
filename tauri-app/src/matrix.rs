//! HUB75 LED matrix canvas renderer and Yew component.

use core::f64::consts::PI;

use basketball::{Game, Quarter, Scorable, State};
use hub75::{Color, DisplayBuffer, RED};
use web_sys::wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{Html, Properties, function_component, html, use_effect_with, use_mut_ref, use_node_ref};

/// Width of the simulated HUB75 LED matrix in pixels.
pub const MATRIX_WIDTH: usize = 64;

/// Height of the simulated HUB75 LED matrix in pixels.
pub const MATRIX_HEIGHT: usize = 32;

/// Half-height (scan rows) of the simulated HUB75 LED matrix.
pub const MATRIX_HALF_HEIGHT: usize = 16;

/// Canvas pixel pitch per simulated LED.
pub const PIXEL_PITCH: f64 = 10.0;

/// Total width of the HTML canvas in pixels.
pub const CANVAS_WIDTH: u32 = (MATRIX_WIDTH as u32) * (PIXEL_PITCH as u32);

/// Total height of the HTML canvas in pixels.
pub const CANVAS_HEIGHT: u32 = (MATRIX_HEIGHT as u32) * (PIXEL_PITCH as u32);

/// Reusable string buffers for formatting scoreboard text onto a [`DisplayBuffer`].
#[derive(Default)]
pub struct ScoreboardLayout {
    home_buffer: String,
    guest_buffer: String,
    quarter_buffer: String,
    clock_buffer: String,
}

impl ScoreboardLayout {
    /// Creates a new [`ScoreboardLayout`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Formats `game` state and renders it into `display` using `color`.
    pub fn render<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>(
        &mut self,
        display: &mut DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>,
        game: &Game,
        color: Color,
    ) {
        use std::fmt::Write;

        self.home_buffer.clear();
        self.guest_buffer.clear();
        self.quarter_buffer.clear();
        self.clock_buffer.clear();

        let total_seconds = game.regulation_millis() / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;

        let scores = game.scores();
        let _ = write!(self.home_buffer, "{:03}", scores.home);
        let _ = write!(self.guest_buffer, "{:03}", scores.away);
        let _ = write!(self.quarter_buffer, "{:?}", game.quarter());
        let _ = write!(self.clock_buffer, "{minutes:02}:{seconds:02}");

        self.draw_text_layout(
            display,
            color,
            &self.home_buffer,
            &self.guest_buffer,
            &self.quarter_buffer,
            &self.clock_buffer,
        );
    }

    /// Clears `display`, draws the four scoreboard text fields, and prepares bitplanes.
    pub fn draw_text_layout<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>(
        &self,
        display: &mut DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>,
        color: Color,
        home: &str,
        guest: &str,
        quarter: &str,
        clock: &str,
    ) {
        display.clear();

        let home_x = 3;
        let guest_x = (WIDTH as i32) - 26;
        let quarter_x = 3;
        let clock_x = 22;

        display.draw_text(home, home_x, 3, color);
        display.draw_text(guest, guest_x, 3, color);
        display.draw_text(quarter, quarter_x, 20, color);
        display.draw_text(clock, clock_x, 20, color);

        display.prepare_bitplanes();
    }
}

/// Renders `display` onto an [`HtmlCanvasElement`] using the buffer's configured brightness.
pub fn render_hub75_to_canvas(
    canvas: &HtmlCanvasElement,
    display: &DisplayBuffer<MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_HALF_HEIGHT>,
) {
    render_hub75_to_canvas_with_brightness(canvas, display, display.brightness());
}

/// Renders `display` onto an [`HtmlCanvasElement`] at `brightness_percent` (`0..=100`).
pub fn render_hub75_to_canvas_with_brightness(
    canvas: &HtmlCanvasElement,
    display: &DisplayBuffer<MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_HALF_HEIGHT>,
    brightness_percent: u8,
) {
    let Ok(Some(context_object)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(context) = context_object.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    // Matrix background / bezel
    context.set_fill_style_str("#05070a");
    context.fill_rect(0.0, 0.0, f64::from(CANVAS_WIDTH), f64::from(CANVAS_HEIGHT));

    let led_radius = PIXEL_PITCH * 0.36;
    let brightness_factor = f64::from(brightness_percent.min(100)) / 100.0;

    for y in 0..MATRIX_HEIGHT {
        for x in 0..MATRIX_WIDTH {
            let [red, green, blue] = display.frame_buffer[y][x];
            let center_x = (x as f64) * PIXEL_PITCH + (PIXEL_PITCH / 2.0);
            let center_y = (y as f64) * PIXEL_PITCH + (PIXEL_PITCH / 2.0);

            context.begin_path();
            let _ = context.arc(center_x, center_y, led_radius, 0.0, 2.0 * PI);

            if red == 0 && green == 0 && blue == 0 {
                // Inactive SMD RGB LED
                context.set_fill_style_str("#161b24");
                context.fill();
            } else {
                // Active SMD RGB LED with brightness applied
                let adjusted_red = (f64::from(red) * brightness_factor).round() as u8;
                let adjusted_green = (f64::from(green) * brightness_factor).round() as u8;
                let adjusted_blue = (f64::from(blue) * brightness_factor).round() as u8;

                let fill = format!("rgb({adjusted_red}, {adjusted_green}, {adjusted_blue})");
                context.set_fill_style_str(&fill);
                context.fill();

                // Lens core glow scaled with brightness
                context.begin_path();
                let _ = context.arc(center_x, center_y, led_radius * 0.45, 0.0, 2.0 * PI);
                let core_red =
                    (((f64::from(red) + 80.0).min(255.0)) * brightness_factor).round() as u8;
                let core_green =
                    (((f64::from(green) + 80.0).min(255.0)) * brightness_factor).round() as u8;
                let core_blue =
                    (((f64::from(blue) + 80.0).min(255.0)) * brightness_factor).round() as u8;
                context.set_fill_style_str(&format!("rgb({core_red}, {core_green}, {core_blue})"));
                context.fill();
            }
        }
    }
}

/// Properties for the [`Hub75Matrix`] Yew component.
#[derive(Properties, PartialEq)]
pub struct Hub75MatrixProps {
    #[prop_or_default]
    pub game: Option<Game>,
    #[prop_or(0)]
    pub home_score: u16,
    #[prop_or(0)]
    pub away_score: u16,
    #[prop_or(State::Idle)]
    pub state: State,
    #[prop_or(0)]
    pub regulation_millis: u32,
    #[prop_or(Quarter::Q1)]
    pub quarter: Quarter,
    #[prop_or_default]
    pub color_override: Option<Color>,
    #[prop_or(20)]
    pub brightness: u8,
}

/// Yew component that renders a simulated 64x32 HUB75 RGB LED matrix on an HTML canvas.
#[function_component]
pub fn Hub75Matrix(props: &Hub75MatrixProps) -> Html {
    let canvas_ref = use_node_ref();
    let display_buffer =
        use_mut_ref(DisplayBuffer::<MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_HALF_HEIGHT>::default);
    let layout_ref = use_mut_ref(ScoreboardLayout::new);

    let game_option = props.game;
    let home_score = props.home_score;
    let away_score = props.away_score;
    let state = props.state;
    let regulation_millis = props.regulation_millis;
    let quarter = props.quarter;
    let color_override = props.color_override;
    let brightness = props.brightness;

    {
        let canvas_ref = canvas_ref.clone();
        let display_buffer = display_buffer.clone();
        let layout_ref = layout_ref.clone();

        use_effect_with(
            (
                game_option,
                home_score,
                away_score,
                state,
                regulation_millis,
                quarter,
                color_override,
                brightness,
            ),
            move |(
                game_option,
                home_score,
                away_score,
                _state,
                regulation_millis,
                quarter,
                color_override,
                brightness,
            )| {
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    let mut display = display_buffer.borrow_mut();
                    let mut layout = layout_ref.borrow_mut();

                    display.set_brightness(*brightness);
                    let color = color_override.unwrap_or(RED);

                    if let Some(game) = game_option {
                        layout.render(&mut display, game, color);
                    } else {
                        // Construct string layout from props directly if no Game provided
                        let total_seconds = *regulation_millis / 1000;
                        let minutes = total_seconds / 60;
                        let seconds = total_seconds % 60;
                        let home_str = format!("{home_score:03}");
                        let guest_str = format!("{away_score:03}");
                        let quarter_str = format!("{quarter:?}");
                        let clock_str = format!("{minutes:02}:{seconds:02}");

                        layout.draw_text_layout(
                            &mut display,
                            color,
                            &home_str,
                            &guest_str,
                            &quarter_str,
                            &clock_str,
                        );
                    }

                    render_hub75_to_canvas_with_brightness(&canvas, &display, *brightness);
                }
                || ()
            },
        );
    }

    html! {
        <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; width: 100%;">
            <div style="background: #020408; padding: 12px; border-radius: 8px; border: 2px solid #334155; box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.7); aspect-ratio: 2 / 1; max-width: 640px; width: 100%; display: flex; align-items: center; justify-content: center; overflow: hidden;">
                <canvas
                    ref={canvas_ref}
                    width={CANVAS_WIDTH.to_string()}
                    height={CANVAS_HEIGHT.to_string()}
                    style="width: 100%; height: auto; display: block; border-radius: 4px; aspect-ratio: 2 / 1;"
                />
            </div>
        </div>
    }
}
