pub mod draw;
mod snippet_buffer;
use crate::config::{self, ColorConfig, Config};
use crate::system::ClipboardStorage;
use crate::ui::draw::DrawUtil;
use crate::ui::snippet_buffer::SnippetBuffer;
use sdl2::ttf::FontStyle;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    pixels::Color,
    rect::Rect,
    ttf,
};

impl From<ColorConfig> for Color {
    fn from(value: config::ColorConfig) -> Self {
        match value {
            config::ColorConfig::RGB(r, g, b) => Color::RGB(r, g, b),
            config::ColorConfig::RGBA(r, g, b, a) => Color::RGBA(r, g, b, a),
        }
    }
}
// TODO: add cursor position
pub struct DClipWindow {
    draw_util: DrawUtil,
    input_buffer: String,
    cursor_pos: usize,
    snippets: ClipboardStorage,
    selected_index: usize,
    paste_text: Option<String>,
    snippet_buffer: SnippetBuffer,
    needs_update: bool,
}

impl DClipWindow {
    pub fn new(copying: bool, config: Config) -> Self {
        let snippets = ClipboardStorage::load().expect("Failed to load snippets.json");
        let snippet_buffer = SnippetBuffer::new();
        let draw_util = DrawUtil::new(copying, config);

        DClipWindow {
            draw_util,
            input_buffer: String::from(""),
            cursor_pos: 0,
            snippets,
            selected_index: 0,
            paste_text: None,
            snippet_buffer,
            needs_update: true,
        }
    }

    pub fn launch(&mut self) -> Option<String> {
        let ttf = ttf::init().expect("Failed to initialize TTF.");
        let mut font = self.draw_util.initialize_font(&ttf);

        let mut event_pump = self
            .draw_util
            .context
            .event_pump()
            .expect("Failed to create event pump.");

        self.draw_util.text.start();
        'running: loop {
            self.draw_util
                .canvas
                .set_draw_color(self.draw_util.user_config.background); // background color
            self.draw_util
                .canvas
                .set_blend_mode(sdl2::render::BlendMode::Blend);
            self.draw_util.canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode, keymod, ..
                    } => {
                        if let Some(key) = keycode {
                            match key {
                                Keycode::Backspace => {
                                    if self.cursor_pos != 0 {
                                        self.cursor_pos -= 1;
                                        let _ = self.input_buffer.remove(self.cursor_pos);
                                        self.needs_update = true;
                                    }
                                }
                                Keycode::D => {
                                    if keymod == Mod::LCTRLMOD
                                        && self.snippets.get_entries().len() != 0
                                    {
                                        let _ = self.snippets.remove_entry(self.selected_index);
                                        let _ = self.selected_index = 0;
                                        self.needs_update = true;
                                    }
                                }
                                Keycode::Left => {
                                    if self.cursor_pos != 0 {
                                        self.cursor_pos -= 1;
                                    }
                                }
                                Keycode::Right => {
                                    if self.cursor_pos != self.input_buffer.chars().count() {
                                        self.cursor_pos += 1;
                                    }
                                }
                                Keycode::UP => {
                                    if self.draw_util.copied_text.is_none() {
                                        if self.selected_index > 0 {
                                            self.selected_index -= 1;
                                        } else {
                                            self.selected_index = self
                                                .snippet_buffer
                                                .snippet_indices
                                                .len()
                                                .saturating_sub(1);
                                        }
                                    }
                                }
                                Keycode::DOWN => {
                                    if self.draw_util.copied_text.is_none() {
                                        if self.selected_index
                                            < self
                                                .snippet_buffer
                                                .snippet_indices
                                                .len()
                                                .saturating_sub(1)
                                        {
                                            self.selected_index += 1;
                                        } else {
                                            self.selected_index = 0;
                                        }
                                    }
                                }
                                Keycode::Return => {
                                    // copy route
                                    if self.draw_util.copied_text.is_some() {
                                        let nickname = Some(self.input_buffer.clone());
                                        let _ = self.snippets.add_entry(
                                            self.draw_util.copied_text.clone().unwrap(),
                                            nickname,
                                        );

                                        break 'running;
                                    } else {
                                        let filtered_select = self.snippet_buffer.snippet_indices
                                            [self.selected_index];
                                        if let Some(selected_snippet) =
                                            self.snippets.get_entries().get(filtered_select)
                                        {
                                            self.paste_text =
                                                Some(selected_snippet.content.clone());
                                            break 'running;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::TextInput { text, .. } => {
                        // get the first char of the string (text)
                        self.input_buffer
                            .insert(self.cursor_pos, text.chars().next().unwrap());
                        self.cursor_pos += 1;
                        self.needs_update = true;
                    }
                    _ => {}
                }
            }

            if self.needs_update {
                let _ = self
                    .snippet_buffer
                    .update(&self.snippets, &self.input_buffer);
                self.needs_update = false;
            }

            if !self.input_buffer.is_empty() {
                // Rendering input_buffer text
                let _ = self
                    .draw_util
                    .draw_text(&font, &self.input_buffer, 12, 12, None, None);
            } else {
                let mut placeholder = String::from("Type to search");
                if self.draw_util.copied_text.is_some() {
                    placeholder = String::from("Add a nickname");
                }

                let _ = self
                    .draw_util
                    .draw_text(&font, &placeholder, 12, 12, None, None);
            }

            if self.draw_util.copied_text.is_some() {
                // render the selected text persistently
                if let Some(selected) = self.draw_util.copied_text.clone() {
                    let _ = self
                        .draw_util
                        .draw_text(&font, &selected, 12, 60, None, None);
                }
            } else {
                let active_snippets = &self.snippet_buffer.snippet_indices;
                let snippets_count = active_snippets.len();
                let current_page = self.selected_index / 5;
                let start_index = current_page * 5;
                let end_index = std::cmp::min(start_index + 5, snippets_count);
                let mut y: i32 = 50;
                for (page_index, &snippet_index) in
                    active_snippets[start_index..end_index].iter().enumerate()
                {
                    let snippet = &self.snippets.get_entries()[snippet_index];
                    let global_index = start_index + page_index;

                    // truncate longer snippets
                    let rendered_snippet = if snippet.content.len() > 50 {
                        format!("{}...", &snippet.content[..50])
                    } else {
                        snippet.content.clone()
                    };

                    font.set_style(FontStyle::BOLD);
                    let _ = self.draw_util.render_list(
                        &font,
                        global_index == self.selected_index,
                        y,
                        &rendered_snippet,
                        &self.selected_index,
                        &self.snippet_buffer.snippet_indices.len(),
                    );

                    y += 35;
                }
            }

            self.draw_util.canvas.present();
        }
        self.paste_text.clone()
    }
}
