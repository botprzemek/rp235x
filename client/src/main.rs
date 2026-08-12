use gloo_timers::callback::Interval;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct Action {
    team: Team,
    points: u8,
}

#[derive(Clone, Copy, PartialEq)]
enum Team {
    Home,
    Away,
}

const IBM_FONT_7X9: [[u8; 9]; 10] = [
    [
        0b01111100, 0b11000110, 0b11001110, 0b11011110, 0b11110110, 0b11100110, 0b11000110,
        0b11000110, 0b01111100,
    ], // '0'
    [
        0b00011000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
        0b01111110, 0b00000000,
    ], // '1'
    [
        0b01111100, 0b11000110, 0b00000110, 0b00011100, 0b00110000, 0b01100000, 0b11000000,
        0b11111110, 0b00000000,
    ], // '2'
    [
        0b01111100, 0b11000110, 0b00000110, 0b00111100, 0b00000110, 0b00000110, 0b11000110,
        0b01111100, 0b00000000,
    ], // '3'
    [
        0b00001100, 0b00011100, 0b00110100, 0b01100100, 0b11000100, 0b11111110, 0b00000100,
        0b00000100, 0b00000000,
    ], // '4'
    [
        0b11111110, 0b11000000, 0b11000000, 0b11111100, 0b00000110, 0b00000110, 0b11000110,
        0b01111100, 0b00000000,
    ], // '5'
    [
        0b00111100, 0b01100000, 0b11000000, 0b11111100, 0b11000110, 0b11000110, 0b11000110,
        0b01111100, 0b00000000,
    ], // '6'
    [
        0b11111110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01100000, 0b01100000,
        0b01100000, 0b00000000,
    ], // '7'
    [
        0b01111100, 0b11000110, 0b11000110, 0b01111100, 0b11000110, 0b11000110, 0b11000110,
        0b01111100, 0b00000000,
    ], // '8'
    [
        0b01111100, 0b11000110, 0b11000110, 0b11000110, 0b01111110, 0b00000110, 0b01100110,
        0b00111100, 0b00000000,
    ], // '9'
];

fn get_ibm_pixel(digit: usize, px: usize, py: usize) -> bool {
    if digit > 9 || py > 8 || px > 6 {
        return false;
    }
    let row_byte = IBM_FONT_7X9[digit][py];
    (row_byte & (1 << (7 - px as u8))) != 0
}

#[function_component(App)]
fn app() -> Html {
    let home_score = use_state(|| 0u32);
    let away_score = use_state(|| 0u32);
    let seconds_elapsed = use_state(|| 0u32);
    let is_running = use_state(|| false);
    let action_stack = use_state(|| Vec::<Action>::new());

    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let home_score = *home_score;
        let away_score = *away_score;

        use_effect_with((home_score, away_score), move |_| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let context = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();

                let width = canvas.width() as usize;
                let height = canvas.height() as usize;

                context.set_fill_style(&wasm_bindgen::JsValue::from_str("#050505"));
                context.fill_rect(0.0, 0.0, width as f64, height as f64);

                let home_str = format!("{:03}", home_score);
                let guest_str = format!("{:03}", away_score);

                let r1_start_y = 2;
                let r1_start_x = 0;
                let block_width = home_str.len() * 8;
                let gap_px = 17;

                context.set_fill_style(&wasm_bindgen::JsValue::from_str("#FF0000"));

                for y in 0..height {
                    for x in 0..width {
                        if y >= r1_start_y && y < r1_start_y + 9 && x >= r1_start_x {
                            let local_x = x - r1_start_x;
                            let mut r1 = false;

                            if local_x < block_width {
                                let char_idx = local_x / 8;
                                let pixel_x = local_x % 8;
                                let pixel_y = y - r1_start_y;

                                if char_idx < home_str.len() && pixel_x < 7 {
                                    let c = home_str.as_bytes()[char_idx] - b'0';
                                    if c <= 9 {
                                        r1 = get_ibm_pixel(c as usize, pixel_x, pixel_y);
                                    }
                                }
                            } else if local_x >= block_width + gap_px {
                                let right_local_x = local_x - (block_width + gap_px);
                                let char_idx = right_local_x / 8;
                                let pixel_x = right_local_x % 8;
                                let pixel_y = y - r1_start_y;

                                if char_idx < guest_str.len() && pixel_x < 7 {
                                    let c = guest_str.as_bytes()[char_idx] - b'0';
                                    if c <= 9 {
                                        r1 = get_ibm_pixel(c as usize, pixel_x, pixel_y);
                                    }
                                }
                            }

                            if r1 {
                                context.fill_rect(x as f64, y as f64, 1.0, 1.0);
                            }
                        }
                    }
                }
            }
            || {}
        });
    }

    {
        let is_running = *is_running;
        let seconds_elapsed = seconds_elapsed.clone();
        use_effect_with(is_running, move |is_running| {
            let mut interval_handle = None;
            if *is_running {
                let handle = Interval::new(1000, move || {
                    seconds_elapsed.set(*seconds_elapsed + 1);
                });
                interval_handle = Some(handle);
            }
            move || {
                drop(interval_handle);
            }
        });
    }

    let add_score = {
        let home_score = home_score.clone();
        let away_score = away_score.clone();
        let action_stack = action_stack.clone();
        Callback::from(move |(team, points): (Team, u8)| {
            let mut stack = (*action_stack).clone();
            stack.push(Action { team, points });
            action_stack.set(stack);

            match team {
                Team::Home => home_score.set(*home_score + points as u32),
                Team::Away => away_score.set(*away_score + points as u32),
            }
        })
    };

    let undo = {
        let home_score = home_score.clone();
        let away_score = away_score.clone();
        let action_stack = action_stack.clone();
        Callback::from(move |_| {
            let mut stack = (*action_stack).clone();
            if let Some(last) = stack.pop() {
                action_stack.set(stack);
                match last.team {
                    Team::Home => home_score.set((*home_score).saturating_sub(last.points as u32)),
                    Team::Away => away_score.set((*away_score).saturating_sub(last.points as u32)),
                }
            }
        })
    };

    let toggle_play = {
        let is_running = is_running.clone();
        Callback::from(move |_| {
            is_running.set(!*is_running);
        })
    };

    let reset_game = {
        let home_score = home_score.clone();
        let away_score = away_score.clone();
        let seconds_elapsed = seconds_elapsed.clone();
        let is_running = is_running.clone();
        let action_stack = action_stack.clone();
        Callback::from(move |_| {
            is_running.set(false);
            home_score.set(0);
            away_score.set(0);
            seconds_elapsed.set(0);
            action_stack.set(Vec::new());
        })
    };

    let start_stop_bg = if *is_running { "#c62828" } else { "#2e7d32" };
    let start_stop_text = if *is_running { "STOP" } else { "START" };

    html! {
        <div style="background-color: #121212; color: #e0e0e0; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; margin: 0;">
            <h1>{ "LED Matrix Scoreboard Simulator (64x32)" }</h1>
            <div style="background-color: #000; padding: 15px; border-radius: 12px; box-shadow: 0 8px 24px rgba(0,0,0,0.8); border: 2px solid #333; margin-bottom: 20px;">
                <canvas ref={canvas_ref} width="64" height="32" style="display: block; width: 512px; height: 256px; image-rendering: pixelated; image-rendering: crisp-edges;"></canvas>
            </div>

            <div style="display: flex; flex-direction: column; background: #1e1e1e; padding: 20px; border-radius: 10px; border: 1px solid #333; width: 500px;">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <div style="display: flex; gap: 5px;">
                        <button class="btn-blue" onclick={let add_score = add_score.clone(); move |_| add_score.emit((Team::Home, 1))}>{ "+1" }</button>
                        <button class="btn-blue" onclick={let add_score = add_score.clone(); move |_| add_score.emit((Team::Home, 2))}>{ "+2" }</button>
                        <button class="btn-blue" onclick={let add_score = add_score.clone(); move |_| add_score.emit((Team::Home, 3))}>{ "+3" }</button>
                    </div>

                    <div style="display: flex; gap: 5px;">
                        <button class="btn-red" onclick={let add_score = add_score.clone(); move |_| add_score.emit((Team::Away, 1))}>{ "+1" }</button>
                        <button class="btn-red" onclick={let add_score = add_score.clone(); move |_| add_score.emit((Team::Away, 2))}>{ "+2" }</button>
                        <button class="btn-red" onclick={let add_score = add_score.clone(); move |_| add_score.emit((Team::Away, 3))}>{ "+3" }</button>
                    </div>
                </div>

                <div style="display: flex; justify-content: space-between; gap: 8px; margin-top: 15px;">
                    <button class="btn-action" onclick={undo}>{ "Cofnij (Undo)" }</button>
                    <button onclick={toggle_play} style={format!("background-color: {}; color: white; border: 1px solid #555; padding: 8px; flex: 1; border-radius: 4px; cursor: pointer; font-weight: bold;", start_stop_bg)}>{ start_stop_text }</button>
                    <button onclick={reset_game} style="background-color: #424242; color: white; border: 1px solid #555; padding: 8px; flex: 1; border-radius: 4px; cursor: pointer; font-weight: bold;">{ "RESET" }</button>
                </div>
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
