use sdl2::render::{Texture, TextureCreator, TextureQuery};
use sdl2::rect::Point;
use sdl2::ttf::Font;
use sdl2::pixels::Color;

pub struct Label<'ttf_module, 'rwops, 'tc, T> {
    text: String,
    font: &'ttf_module Font<'ttf_module, 'rwops>,
    pub position: Point,
    color: Color,
    width: u32,
    height: u32,
    texture_creator: &'tc TextureCreator<T>,
    cached_texture: Option<Texture<'tc>>
}

impl<'ttf_module, 'rwops, 'tc, T> Label<'ttf_module, 'rwops, 'tc, T> {
    pub fn new(text: String, 
        font: &'ttf_module Font<'ttf_module, 'rwops>, 
        position: Point, 
        color: Color, 
        texture_creator: &'tc TextureCreator<T>) -> Result<Self, String> {
        let mut label = Label {
            text: text,
            font: &font,
            position: position,
            color: color,
            width: 0,
            height: 0,
            texture_creator: texture_creator,
            cached_texture: None
        };
        label.create_cache()?;
        Ok(label)
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.clear_cache();
    }

    fn clear_cache(&mut self) {
        self.cached_texture = None
    }

    pub fn texture(&mut self) -> Result<&Texture<'tc>, String> {
        match self.cached_texture {
            Some(ref texture) => Ok(&texture),
            None => Ok(self.create_cache()?)
        }
    }

    fn create_texture(&self, 
        text: &str, 
        font: &Font, 
        color: &Color, 
        texture_creator: &'tc TextureCreator<T>) -> Result<Texture<'tc>, String> {
        let surface = font.render(text)
            .blended(*color).map_err(|e| e.to_string())?;
        let texture = texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        Ok(texture)
    }

    fn create_cache(&mut self) -> Result<&Texture<'tc>, String> {
        let texture = self.create_texture(
            &self.text, 
            &self.font, 
            &self.color, 
            &self.texture_creator
        )?;
        let TextureQuery { width, height, .. } = texture.query();
        self.width = width;
        self.height = height;
        self.cached_texture = Some(texture);
        Ok(self.cached_texture.as_ref().unwrap())
    }

    pub fn width(&mut self) -> Result<u32, String> {
        if !self.cached_texture.is_some() {
            self.create_cache()?;
        }
        Ok(self.width)
    }

    pub fn height(&mut self) -> Result<u32, String> {
        if !self.cached_texture.is_some() {
            self.create_cache()?;
        }
        Ok(self.height)
    }
}