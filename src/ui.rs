// TODO:
// fuzzy finder [ DONE ]
// line wrapping (copy) [ DONE ]
// elipses (...) in paste [ DONE ]
// user config [ DONE ]
// styling (dividing line, transparent placeholder text) [ DONE ]
use crate::config::{self, ColorConfig, Config};
use crate::system::ClipboardStorage;
use font_kit::{handle::Handle, source::SystemSource};
use sdl2::{
    clipboard::ClipboardUtil,
    event::Event,
    keyboard::{Keycode, Mod, TextInputUtil},
    pixels::Color,
    rect::Rect,
    render::Canvas,
    ttf,
    video::Window,
    Sdl, VideoSubsystem,
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
    context: Sdl,
    canvas: Canvas<Window>,
    ttf: ttf::Sdl2TtfContext,
    input_buffer: String,
    text: TextInputUtil,
    cursor_pos: usize,
    copied_text: Option<String>,
    snippets: ClipboardStorage,
    selected_index: usize,
    paste_text: Option<String>,
    filtered_snippets: Vec<usize>,
    needs_update: bool,
    user_config: Config,
}

impl DClipWindow {
    pub fn new(copying: bool, config: Config) -> Self {
        let context = sdl2::init().expect("Failed to create sdl2 context.");

        let video: VideoSubsystem = context
            .video()
            .expect("Failed to initialize sdl2 video subsystem.");

        let clipboard: ClipboardUtil = video.clipboard();

        let copied_text = if copying && clipboard.has_primary_selection_text() {
            Some(clipboard.primary_selection_text().unwrap())
        } else {
            None
        };

        let mut window = video
            .window("", config.width as u32, config.height as u32)
            .position_centered()
            .borderless()
            .build()
            .expect("Failed to create window.");
        let _ = window.set_opacity(0.5);

        let mut canvas = window
            .into_canvas()
            .build()
            .expect("Failed to create canvas.");
        let _ = canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

        let ttf = ttf::init().expect("Failed to retrieve ttf context.");
        let text = video.text_input();

        let snippets = ClipboardStorage::load().expect("Failed to load snippets.json");

        DClipWindow {
            context,
            canvas,
            ttf,
            input_buffer: String::from(""),
            text,
            cursor_pos: 0,
            copied_text,
            snippets,
            selected_index: 0,
            paste_text: None,
            filtered_snippets: Vec::new(),
            needs_update: true,
            user_config: config,
        }
    }

    fn fuzzy_find(&self, content: &str) -> bool {
        let query = &self.input_buffer.to_lowercase();
        let content_lower = content.to_lowercase();
        let mut content_chars = content_lower.chars().peekable();

        for qc in query.chars() {
            loop {
                match content_chars.next() {
                    Some(sc) if sc == qc => break,
                    Some(_) => continue,
                    None => return false,
                }
            }
        }
        true
    }

    pub fn launch(&mut self) -> Option<String> {
        let creator = self.canvas.texture_creator();

        let source = SystemSource::new();
        let fonts = source
            .all_fonts()
            .expect("Failed to retrieve list of fonts.");
        let selected_font = fonts.get(0).expect("No fonts found.");
        let mut font_path = String::from("");
        if let Handle::Path { path, .. } = selected_font {
            font_path = path.to_str().expect("Failed to get font path.").to_string();
        }

        let mut font = self
            .ttf
            .load_font(font_path, self.user_config.font_size)
            .expect("Failed to load font.");

        font.set_style(sdl2::ttf::FontStyle::BOLD);

        let mut event_pump = self
            .context
            .event_pump()
            .expect("Failed to create event pump.");

        self.text.start();
        'running: loop {
            self.canvas.set_draw_color(self.user_config.background); // background color
            self.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            self.canvas.clear();
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
                                        let _ = self.selected_index = 0;
                                        let _ = self.snippets.remove_entry(self.selected_index);
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
                                    if self.copied_text.is_none() {
                                        if self.selected_index > 0 {
                                            self.selected_index -= 1;
                                        } else {
                                            self.selected_index =
                                                self.filtered_snippets.len().saturating_sub(1);
                                        }
                                    }
                                }
                                Keycode::DOWN => {
                                    if self.copied_text.is_none() {
                                        if self.selected_index
                                            < self.filtered_snippets.len().saturating_sub(1)
                                        {
                                            self.selected_index += 1;
                                        } else {
                                            self.selected_index = 0;
                                        }
                                    }
                                }
                                Keycode::Return => {
                                    // copy route
                                    if self.copied_text.is_some() {
                                        let nickname = Some(self.input_buffer.clone());
                                        let _ = self
                                            .snippets
                                            .add_entry(self.copied_text.clone().unwrap(), nickname);

                                        break 'running;
                                    } else {
                                        if let Some(&snippet_index) =
                                            self.filtered_snippets.get(self.selected_index)
                                        {
                                            if let Some(selected_snippet) =
                                                self.snippets.get_entries().get(snippet_index)
                                            {
                                                self.paste_text =
                                                    Some(selected_snippet.content.clone());
                                                break 'running;
                                            }
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
                let mut matches: Vec<(usize, bool)> = Vec::new();
                for (i, snippet) in self.snippets.get_entries().iter().enumerate() {
                    let nickname_match = snippet
                        .nickname
                        .as_ref()
                        .map_or(false, |n| self.fuzzy_find(n));

                    let content_match = self.fuzzy_find(&snippet.content);

                    if nickname_match || content_match {
                        matches.push((i, nickname_match));
                    }
                }

                // Prioritize nickname search
                matches.sort_by(|a, b| b.1.cmp(&a.1));

                self.filtered_snippets = matches.into_iter().map(|(i, _)| i).collect();

                self.selected_index = self
                    .selected_index
                    .min(self.filtered_snippets.len().saturating_sub(1));
                self.needs_update = false;
            }

            if !self.input_buffer.is_empty() {
                // TODO: refactor out all the text drawing functions
                // render input_buffer text

                let surface = font
                    .render(&self.input_buffer)
                    .blended(self.user_config.input_color) // input text color
                    .expect("Failed to render text.");

                let texture = creator
                    .create_texture_from_surface(&surface)
                    .expect("Failed to create texture.");

                // render cursor
                let rect = Rect::new(12, 12, surface.width(), surface.height());
                let (text_width, _) = font
                    .size_of(&self.input_buffer[0..self.cursor_pos])
                    .unwrap();
                let cursor = Rect::new(12 + text_width as i32, rect.y, 2, font.height() as u32);
                self.canvas.set_draw_color(self.user_config.cursor);
                let _ = self.canvas.fill_rect(cursor);

                let _ = self
                    .canvas
                    .copy(&texture, None, Some(rect))
                    .expect("Failed to copy to canvas.");
            } else {
                let mut placeholder = String::from("Type to search");
                if self.copied_text.is_some() {
                    placeholder = String::from("Add a nickname");
                }

                let surface = font
                    .render(&placeholder)
                    .blended(Color::RGBA(255, 255, 255, 90)) // placeholder text color
                    .expect("Failed to render text.");

                let texture = creator
                    .create_texture_from_surface(&surface)
                    .expect("Failed to create texture.");

                let rect = Rect::new(12, 12, surface.width(), surface.height());

                let cursor = Rect::new(12, 12, 2, font.height() as u32);
                let _ = self.canvas.fill_rect(cursor);
                let _ = self
                    .canvas
                    .copy(&texture, None, Some(rect))
                    .expect("Failed to copy to canvas.");
            }

            if self.copied_text.is_some() {
                // render the selected text persistently
                if let Some(selected) = &self.copied_text {
                    let copy_surface = font
                        .render(selected)
                        .blended_wrapped(Color::RGBA(255, 255, 255, 90), 900) // copy text color
                        .expect("Failed to render text.");

                    let texture = creator
                        .create_texture_from_surface(&copy_surface)
                        .expect("Failed to create texture.");

                    let rect = Rect::new(12, 60, copy_surface.width(), copy_surface.height());

                    let _ = self
                        .canvas
                        .copy(&texture, None, Some(rect))
                        .expect("Failed to copy to canvas.");
                }
            } else {
                let active_snippets = &self.filtered_snippets;
                let snippets_count = active_snippets.len();
                let current_page = self.selected_index / 5;
                let start_index = current_page * 5;
                let end_index = std::cmp::min(start_index + 5, snippets_count);
                let mut y: i32 = 50;
                for (page_index, &snippet_index) in self.filtered_snippets[start_index..end_index]
                    .iter()
                    .enumerate()
                {
                    let snippet = &self.snippets.get_entries()[snippet_index];
                    let global_index = start_index + page_index;
                    let color = if global_index == self.selected_index {
                        self.user_config.selected_color // selected text color
                    } else {
                        self.user_config.unselected_color // unselected text color
                    };

                    // truncate longer snippets
                    let rendered_snippet = if snippet.content.len() > 50 {
                        format!("{}...", &snippet.content[..50])
                    } else {
                        snippet.content.clone()
                    };

                    let copy_surface = font
                        .render(&rendered_snippet)
                        .blended(color)
                        .expect("Failed to render text.");

                    let texture = creator
                        .create_texture_from_surface(&copy_surface)
                        .expect("Failed to create texture.");

                    let highlighter = Rect::new(0, y - 5, 1000, copy_surface.height() + 5);

                    self.canvas
                        .set_draw_color(if global_index == self.selected_index {
                            Color::RGB(80, 75, 56) // selected highlight color
                        } else {
                            Color::RGB(60, 56, 42) // unselected highlight color
                        });
                    self.canvas.fill_rect(highlighter).unwrap();

                    let rect = Rect::new(12, y, copy_surface.width(), copy_surface.height());
                    let _ = self
                        .canvas
                        .copy(&texture, None, Some(rect))
                        .expect("Failed to copy to canvas.");

                    y += 35;

                    let index_label = format!(
                        "{}/{}",
                        &self.selected_index.saturating_add(1),
                        &self.filtered_snippets.len()
                    );

                    let copy_surface = font
                        .render(&index_label)
                        .blended(self.user_config.unselected_color)
                        .expect("Failed to render text.");

                    let texture = creator
                        .create_texture_from_surface(&copy_surface)
                        .expect("Failed to create texture.");

                    let label_rect = Rect::new(
                        1000 - copy_surface.width() as i32 - 10,
                        5,
                        copy_surface.width(),
                        copy_surface.height() + 5,
                    );
                    let _ = self
                        .canvas
                        .copy(&texture, None, Some(label_rect))
                        .expect("Failed to copy to canvas.");
                }
            }

            self.canvas.present();
        }
        self.paste_text.clone()
    }
}
