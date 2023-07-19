use eframe::emath;
use std::cmp;

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl From<emath::Vec2> for Pos {
    fn from(pos: emath::Vec2) -> Self {
        Self {
            x: pos.x as i32,
            y: pos.y as i32,
        }
    }
}

impl From<emath::Pos2> for Pos {
    fn from(pos: emath::Pos2) -> Self {
        Self {
            x: pos.x as i32,
            y: pos.y as i32,
        }
    }
}

impl From<Pos> for emath::Pos2 {
    fn from(pos: Pos) -> Self {
        Self {
            x: pos.x as f32,
            y: pos.y as f32,
        }
    }
}

impl From<Pos> for (isize, isize) {
    fn from(pos: Pos) -> Self {
        (pos.x as isize, pos.y as isize)
    }
}

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

impl Size {
    pub fn is_valid(&self) -> bool {
        self.w > 0 && self.h > 0
    }
}

impl From<emath::Vec2> for Size {
    fn from(size: emath::Vec2) -> Self {
        Self {
            w: size.x.round() as u32,
            h: size.y.round() as u32,
        }
    }
}

impl From<(usize, usize)> for Size {
    fn from((x, y): (usize, usize)) -> Self {
        Self {
            w: x as u32,
            h: y as u32,
        }
    }
}

impl From<Size> for emath::Vec2 {
    fn from(size: Size) -> Self {
        Self {
            x: size.w as f32,
            y: size.h as f32,
        }
    }
}

impl From<Size> for (usize, usize) {
    fn from(size: Size) -> Self {
        (size.w as usize, size.h as usize)
    }
}

#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct Rect {
    pub pos: Pos,
    pub size: Size,
}

impl Rect {
    pub fn scaled(&self, scale: f32) -> Self {
        Self {
            pos: Pos {
                x: (self.pos.x as f32 * scale) as i32,
                y: (self.pos.y as f32 * scale) as i32,
            },
            size: Size {
                w: (self.size.w as f32 * scale).round() as u32,
                h: (self.size.h as f32 * scale).round() as u32,
            },
        }
    }

    pub fn fitted(&self, size: Size) -> Self {
        let max_x = self.pos.x as u32 + self.size.w;
        let x = if self.pos.x < 0 {
            0
        } else if max_x > size.w {
            let d = (max_x - size.w) as i32;
            cmp::max(0, self.pos.x - d)
        } else {
            self.pos.x
        };

        let w = if max_x > size.w {
            size.w - self.pos.x as u32
        } else {
            self.size.w
        };

        let max_y = self.pos.y as u32 + self.size.h;
        let y = if self.pos.y < 0 {
            0
        } else if max_y > size.h {
            let d = (max_y - size.h) as i32;
            cmp::max(0, self.pos.y - d)
        } else {
            self.pos.y
        };

        let h = if max_y > size.h {
            size.h - self.pos.y as u32
        } else {
            self.size.h
        };

        Self {
            pos: Pos { x, y },
            size: Size { w, h },
        }
    }
}

impl From<Rect> for emath::Rect {
    fn from(rect: Rect) -> Self {
        Self::from_min_size(rect.pos.into(), rect.size.into())
    }
}
