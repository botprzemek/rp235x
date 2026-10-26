use basketball::{Game, Quarter, Scorable, State};
use core::f64::consts::PI;
use hub75::{Color, DisplayBuffer, RED};
use web_sys::wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

pub const MATRIX_WIDTH: usize = 64;
pub const MATRIX_HEIGHT: usize = 32;
pub const MATRIX_HALF_HEIGHT: usize = 16;
pub const PIXEL_PITCH: f64 = 10.0;
pub const CANVAS_WIDTH: u32 = (MATRIX_WIDTH as u32) * (PIXEL_PITCH as u32);
pub const CANVAS_HEIGHT: u32 = (MATRIX_HEIGHT as u32) * (PIXEL_PITCH as u32);

#[derive(Default)]
pub struct ScoreboardLayout {
    home_buf: String,
    guest_buf: String,
    quarter_buf: String,
    clock_buf: String,
}

impl ScoreboardLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render<const WIDTH: usize, const HEIGHT: usize, const HALF_HEIGHT: usize>(
        &mut self,
        display: &mut DisplayBuffer<WIDTH, HEIGHT, HALF_HEIGHT>,
        game: &Game,
        color: Color,
    ) {
        self.home_buf.clear();
        self.guest_buf.clear();
        self.quarter_buf.clear();
        self.clock_buf.clear();

        let total_seconds = game.regulation_millis() / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;

        use std::fmt::Write;
        let (home_score, away_score) = game.get_scores();
        let _ = write!(self.home_buf, "{:03}", home_score);
        let _ = write!(self.guest_buf, "{:03}", away_score);
        let _ = write!(self.quarter_buf, "{:?}", game.quarter());
        let _ = write!(self.clock_buf, "{:02}:{:02}", minutes, seconds);

        self.draw_text_layout(
            display,
            color,
            &self.home_buf,
            &self.guest_buf,
            &self.quarter_buf,
            &self.clock_buf,
        );
    }

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

pub fn render_hub75_to_canvas(
    canvas: &HtmlCanvasElement,
    display: &DisplayBuffer<MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_HALF_HEIGHT>,
) {
    render_hub75_to_canvas_with_brightness(canvas, display, display.brightness());
}

pub fn render_hub75_to_canvas_with_brightness(
    canvas: &HtmlCanvasElement,
    display: &DisplayBuffer<MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_HALF_HEIGHT>,
    brightness_percent: u8,
) {
    let Ok(Some(ctx_obj)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(ctx) = ctx_obj.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    // Matrix background / bezel
    ctx.set_fill_style_str("#05070a");
    ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH as f64, CANVAS_HEIGHT as f64);

    let led_radius = PIXEL_PITCH * 0.36;
    let b_factor = brightness_percent.min(100) as f64 / 100.0;

    for y in 0..MATRIX_HEIGHT {
        for x in 0..MATRIX_WIDTH {
            let [r, g, b] = display.frame_buffer[y][x];
            let cx = (x as f64) * PIXEL_PITCH + (PIXEL_PITCH / 2.0);
            let cy = (y as f64) * PIXEL_PITCH + (PIXEL_PITCH / 2.0);

            ctx.begin_path();
            let _ = ctx.arc(cx, cy, led_radius, 0.0, 2.0 * PI);

            if r == 0 && g == 0 && b == 0 {
                // Inactive SMD RGB LED
                ctx.set_fill_style_str("#161b24");
                ctx.fill();
            } else {
                // Active SMD RGB LED with brightness applied
                let adj_r = ((r as f64) * b_factor).round() as u8;
                let adj_g = ((g as f64) * b_factor).round() as u8;
                let adj_b = ((b as f64) * b_factor).round() as u8;

                let fill = format!("rgb({}, {}, {})", adj_r, adj_g, adj_b);
                ctx.set_fill_style_str(&fill);
                ctx.fill();

                // Lens core glow scaled with brightness
                ctx.begin_path();
                let _ = ctx.arc(cx, cy, led_radius * 0.45, 0.0, 2.0 * PI);
                let core_r = (((r as f64 + 80.0).min(255.0)) * b_factor).round() as u8;
                let core_g = (((g as f64 + 80.0).min(255.0)) * b_factor).round() as u8;
                let core_b = (((b as f64 + 80.0).min(255.0)) * b_factor).round() as u8;
                ctx.set_fill_style_str(&format!("rgb({}, {}, {})", core_r, core_g, core_b));
                ctx.fill();
            }
        }
    }
}

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

#[function_component]
pub fn Hub75Matrix(props: &Hub75MatrixProps) -> Html {
    let canvas_ref = use_node_ref();
    let display_buffer =
        use_mut_ref(DisplayBuffer::<MATRIX_WIDTH, MATRIX_HEIGHT, MATRIX_HALF_HEIGHT>::default);
    let layout_ref = use_mut_ref(ScoreboardLayout::new);

    let game_opt = props.game;
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
                game_opt,
                home_score,
                away_score,
                state,
                regulation_millis,
                quarter,
                color_override,
                brightness,
            ),
            move |(
                game_opt,
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

                    if let Some(g) = game_opt {
                        layout.render(&mut display, g, color);
                    } else {
                        // Construct string layout from props directly if no Game provided
                        let total_seconds = *regulation_millis / 1000;
                        let minutes = total_seconds / 60;
                        let seconds = total_seconds % 60;
                        let home_str = format!("{:03}", home_score);
                        let guest_str = format!("{:03}", away_score);
                        let quarter_str = format!("{:?}", quarter);
                        let clock_str = format!("{:02}:{:02}", minutes, seconds);

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
