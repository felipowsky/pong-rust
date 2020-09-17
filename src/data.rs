use sdl2::render::Texture;
use sdl2::rect::{Point, Rect};
use crate::gui::Label;

pub struct Sprite<'a> {
    pub texture: &'a Texture<'a>,
    pub rect: Rect
}

pub struct Entity<'a> {
    pub position: Point,
    pub size: (u32, u32),
    pub sprite: Sprite<'a>
}

pub struct Paddle<'a, 'b, 'c, T> {
    entity_index: usize,
    pub movement: i32,
    score: u64,
    pub label: Label<'a, 'b, 'c, T>
}

impl<'a, 'b, 'c, T> Paddle<'a, 'b, 'c, T> {
    pub fn new(entity_index: usize,
        movement: i32,
        score: u64,
        label: Label<'a, 'b, 'c, T>) -> Paddle<'a, 'b, 'c, T> {
            Paddle {
                entity_index: entity_index,
                movement: movement,
                score: score,
                label: label    
            }
    }

    pub fn entity_index(&self) -> usize {
        self.entity_index
    }

    pub fn increase_score(&mut self) {
        self.score += 1;
        self.label.set_text(format!("{}", self.score));
    }
}

pub struct Ball {
    entity_index: usize, 
    pub movement: Point,
}

impl Ball {
    pub fn new(entity_index: usize, movement: Point) -> Self {
        Ball {
            entity_index: entity_index,
            movement: movement
        }
    }

    pub fn entity_index(&self) -> usize {
        self.entity_index
    }
}