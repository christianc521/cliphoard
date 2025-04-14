use crate::config::{ColorConfig, Config};
use font_kit::{handle::Handle, source::SystemSource};
use sdl2::{
    clipboard::ClipboardUtil,
    keyboard::TextInputUtil,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    surface::Surface,
    sys::ttf::TTF_STYLE_BOLD,
    ttf::{Font, FontStyle, Sdl2TtfContext},
    video::{Window, WindowContext},
    Sdl, VideoSubsystem,
};

use super::snippet_buffer;

pub struct DrawUtil {
    pub context: Sdl,
    pub canvas: Canvas<Window>,
    pub copied_text: Option<String>,
    pub text: TextInputUtil,
    pub user_config: Config,
    pub creator: TextureCreator<WindowContext>,
}

impl DrawUtil {
    pub fn new(copying: bool, config: Config) -> DrawUtil {
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
        let creator = canvas.texture_creator();

        let text = video.text_input();

        DrawUtil {
            context,
            canvas,
            copied_text,
            text,
            user_config: config,
            creator,
        }
    }

    pub fn initialize_font<'a>(&self, ttf: &'a Sdl2TtfContext) -> Font<'a, 'static> {
        let source = SystemSource::new();
        let fonts = source
            .all_fonts()
            .expect("Failed to retrieve list of fonts.");
        let selected_font = fonts.get(0).expect("No fonts found.");

        let mut font_path = String::from("");
        if let Handle::Path { path, .. } = selected_font {
            font_path = path.to_str().expect("Failed to get font path.").to_string();
        };

        let font = ttf
            .load_font(font_path, self.user_config.font_size.clone())
            .expect("Failed to load font.");

        font
    }

    pub fn initialize_text(
        &self,
        font: &Font,
        text: &str,
        color: ColorConfig,
    ) -> (Surface, Texture) {
        let surface = font
            .render(text)
            .blended(color) // input text color
            .expect("Failed to render text.");

        let texture = self
            .creator
            .create_texture_from_surface(&surface)
            .expect("Failed to create texture.");

        (surface, texture)
    }

    pub fn render_list(
        &mut self,
        font: &Font,
        selected: bool,
        y: i32,
        snippet: &str,
        index: &usize,
        buffer_count: &usize,
    ) {
        let (text, highlighter) = if selected {
            (
                self.user_config.selected_color,
                self.user_config.highlighted,
            ) // selected highlight color
        } else {
            (
                self.user_config.unselected_color,
                self.user_config.background,
            ) // unselected highlight color
        };

        let surface = font
            .render(snippet)
            .blended(text) // input text color
            .expect("Failed to render text.");

        let texture = self
            .creator
            .create_texture_from_surface(&surface)
            .expect("Failed to create texture.");

        let snippet_rect = Rect::new(12, y, surface.width(), surface.height());

        // Need to draw the highlighter under the snippet text
        self.canvas.set_draw_color(highlighter);
        let highlighter_rect = Rect::new(0, y - 5, 1000, &surface.height() + 5);
        self.canvas
            .fill_rect(highlighter_rect)
            .expect("Failed to render highlighter.");

        let _ = self
            .canvas
            .copy(&texture, None, Some(snippet_rect))
            .expect("Failed to copy to canvas.");

        let index_label = format!("{}/{}", index.saturating_add(1), buffer_count);

        let surface = font
            .render(&index_label)
            .blended(text) // input text color
            .expect("Failed to render text.");

        let texture = self
            .creator
            .create_texture_from_surface(&surface)
            .expect("Failed to create texture.");

        let label_rect = Rect::new(
            1000 - surface.width() as i32 - 10,
            5,
            surface.width(),
            surface.height() + 5,
        );

        let _ = self
            .canvas
            .copy(&texture, None, Some(label_rect))
            .expect("Failed to render index label.");
    }

    pub fn draw_text(
        &mut self,
        font: &Font,
        text: &str,
        x: i32,
        y: i32,
        height: Option<u32>,
        width: Option<u32>,
    ) {
        let surface = font
            .render(text)
            .blended(self.user_config.input_color) // input text color
            .expect("Failed to render text.");

        let texture = self
            .creator
            .create_texture_from_surface(&surface)
            .expect("Failed to create texture.");

        let height = if height.is_none() {
            surface.height()
        } else {
            height.expect("Failed to unwrap height.")
        };

        let width = if width.is_none() {
            surface.width()
        } else {
            width.expect("Failed to unwrap width.")
        };

        let rect = Rect::new(x, y, width, height);

        // render cursor
        //let (text_width, _) = font.size_of(&text[0..self.cursor_pos]).unwrap();
        //let cursor = Rect::new(12 + text_width as i32, rect.y, 2, font.height() as u32);
        //self.canvas.set_draw_color(self.user_config.cursor);

        let _ = self
            .canvas
            .copy(&texture, None, Some(rect))
            .expect("Failed to copy to canvas.");
    }
}
