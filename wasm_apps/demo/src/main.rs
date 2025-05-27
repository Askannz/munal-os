extern crate alloc;

use lazy_static::lazy_static;

use applib::drawing::primitives::draw_rect;
use applib::drawing::text::{Font, RichText, TextJustification, DEFAULT_FONT_FAMILY};
use applib::Color;
use core::cell::OnceCell;
use guestlib::{PixelData, WasmLogger};
use applib::Rect;
use applib::content::TrackedContent;
use applib::uitk::{self, ButtonConfig, ChoiceButtonsConfig, ChoiceConfig, EditableRichText, TextBoxState, UuidProvider};
use applib::{Framebuffer, OwnedPixels};


lazy_static! {
    pub static ref JUSTIF_LEFT_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/justif_left.png"));
    pub static ref JUSTIF_CENTER_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/justif_center.png"));
    pub static ref JUSTIF_RIGHT_ICON: Framebuffer<OwnedPixels> = 
        Framebuffer::from_png(include_bytes!("../icons/justif_right.png"));
}

struct AppState {
    pixel_data: PixelData,
    ui_store: uitk::UiStore,
    uuid_provider: UuidProvider,

    textbox_text: TrackedContent<RichText>,
    textbox_prelude: TrackedContent<RichText>,
    textbox_state: TextBoxState,

    selected_justif: usize,
    selected_color: usize,
    selected_size: usize,

    editing_enabled: bool,
}

static mut APP_STATE: OnceCell<AppState> = OnceCell::new();

static LOGGER: WasmLogger = WasmLogger;
const LOGGING_LEVEL: log::LevelFilter = log::LevelFilter::Debug;

fn main() {}

#[no_mangle]
pub fn init() -> () {

    log::set_max_level(LOGGING_LEVEL);
    log::set_logger(&LOGGER).unwrap();

    let mut uuid_provider = uitk::UuidProvider::new();

    let selected_justif = 0;
    let selected_color = 0;
    let selected_size = 0;

    let textbox_state = {
        let mut tb_state = TextBoxState::new();
        tb_state.justif = get_justif(selected_justif);
        tb_state
    };

    let color = get_color(selected_color);
    let font = get_font(selected_size);    

    let textbox_text = {
        let text = RichText::from_str("pouet\ntralala", color, font, None);
        TrackedContent::new(text, &mut uuid_provider)
    };

    let textbox_prelude = {
        let text = RichText::from_str("Write text here >>>", color, font, None);
        TrackedContent::new(text, &mut uuid_provider)
    };


    let state = AppState {
        pixel_data: PixelData::new(),
        ui_store: uitk::UiStore::new(),
        uuid_provider: UuidProvider::new(),

        textbox_text,
        textbox_prelude,
        textbox_state,
        selected_justif,
        selected_color,
        selected_size,

        editing_enabled: true,
    };
    unsafe {
        APP_STATE
            .set(state)
            .unwrap_or_else(|_| panic!("App already initialized"));
    }
}

#[no_mangle]
pub fn step() {
    let state = unsafe { APP_STATE.get_mut().expect("App not initialized") };

    let time = guestlib::get_time();
    let stylesheet = guestlib::get_stylesheet();
    let input_state = guestlib::get_input_state();
    let Rect { w, h, ..} = guestlib::get_win_rect();


    let mut framebuffer = state.pixel_data.get_framebuffer();

    let mut uitk_context = state.ui_store.get_context(
        &mut framebuffer,
        &stylesheet,
        &input_state,
        &mut state.uuid_provider,
        time
    );

    draw_rect(
        uitk_context.fb,
        &Rect { x0: (w / 2).into(), y0: 0, w: w / 2, h },
        stylesheet.colors.background,
        false
    );

    let mut y = 0;
    let row_h = 100;

    // Justification

    uitk_context.section(
        &Rect { x0: (w / 2).into(), y0: y, w: w / 2, h: row_h },
        "Justification",
        |context, inner_rect| context.layout_box(
            inner_rect,
            0.0, 0.0, 0.4, 0.0,
            |context, inner_rect|         context.choice_buttons_exclusive(
                &ChoiceButtonsConfig {
                    rect: inner_rect.clone(),
                    choices: vec![
                        ChoiceConfig {
                            text: "".to_owned(),
                            icon: Some(&JUSTIF_LEFT_ICON),
                        },
                        ChoiceConfig {
                            text: "".to_owned(),
                            icon: Some(&JUSTIF_CENTER_ICON),
                        },
                        ChoiceConfig {
                            text: "".to_owned(),
                            icon: Some(&JUSTIF_RIGHT_ICON),
                        },
                    ]
                },
                &mut state.selected_justif
            )
        )
    );

    state.textbox_state.justif = get_justif(state.selected_justif);

    y += row_h as i64;

    // Color

    uitk_context.section(
        &Rect { x0: (w / 2).into(), y0: y, w: w / 2, h: row_h },
        "Color",
        |context, inner_rect| context.choice_buttons_exclusive(
            &ChoiceButtonsConfig {
                rect: inner_rect.clone(),
                choices: vec![
                    ChoiceConfig {
                        text: "White".to_owned(),
                        ..Default::default()
                    },
                    ChoiceConfig {
                        text: "Blue".to_owned(),
                        ..Default::default()
                    },
                    ChoiceConfig {
                        text: "Red".to_owned(),
                        ..Default::default()
                    },
                    ChoiceConfig {
                        text: "Green".to_owned(),
                        ..Default::default()
                    },
                ]
            },
            &mut state.selected_color
        )
    );

    let color = get_color(state.selected_color);

    y += row_h as i64;

    // Font

    uitk_context.section(
        &Rect { x0: (w / 2).into(), y0: y, w: w / 2, h: row_h },
        "Font",
        |context, inner_rect| context.choice_buttons_exclusive(
            &ChoiceButtonsConfig {
                rect: inner_rect.clone(),
                choices: vec![
                    ChoiceConfig {
                        text: "12".to_owned(),
                        ..Default::default()
                    },
                    ChoiceConfig {
                        text: "24".to_owned(),
                        ..Default::default()
                    },
                ]
            },
            &mut state.selected_size
        )
    );

    let font = get_font(state.selected_size);

    y += row_h as i64;

    let section_rect = Rect { x0: (w / 2).into(), y0: y, w: w / 2, h: row_h };

    uitk_context.section(&section_rect, "Section", |context, inner_rect| {
        context.button_toggle(
            &ButtonConfig{
                rect: inner_rect.clone(),
                text: "Enable".to_string(),
                ..Default::default()
            },
            &mut state.editing_enabled,
        );
    });



    let text_box_rect = Rect { x0: 0, y0: 0, w: w / 2, h };

    if state.editing_enabled {
        uitk_context.editable_text_box(
            &text_box_rect,
            &mut EditableRichText {
                color,
                font,
                rich_text: &mut state.textbox_text
            },
            &mut state.textbox_state,
            true,
            true,
            Some(&state.textbox_prelude)
        );
    } else {
        uitk_context.text_box(
            &text_box_rect,
            &state.textbox_text,
            &mut state.textbox_state,
            true
        );
    }


    // uitk_context.text_box(
    //     &Rect { x0: (w / 2) as i64, y0: 0, w: w / 2, h },
    //     &state.textbox_text,
    //     &mut state.textbox_2_state,
    //     true
    // );
}

fn get_justif(selected: usize) -> TextJustification {
    match selected {
        0 => TextJustification::Left,
        1 => TextJustification::Center,
        _ => TextJustification::Right,
    }
}

fn get_color(selected: usize) -> Color {
    match selected {
        0 => Color::WHITE,
        1 => Color::BLUE,
        _ => Color::RED,
    }
}

fn get_font(selected: usize) -> &'static Font {
    let size = match selected {
        0 => 12,
        _ => 24,
    };

    DEFAULT_FONT_FAMILY.get_size(size)
}

