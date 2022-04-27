use eframe::egui::ColorImage;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction{
    Left,
    Right,
    Up,
    Down,
    Dontapply,
}

#[derive(Debug, Clone)]
pub struct TaquinSide{
    pub num_line: usize,
    pub num_col: usize,
}


impl Default for TaquinSide{
    fn default() -> Self{
        Self{
            num_line: 5,
            num_col: 5,
        }
    }
}


#[derive(Debug)]
pub struct TaquinCoord{
    pub line: usize,
    pub col: usize,
}

impl Default for TaquinCoord{
    fn default() -> Self{
        Self{
            line: 0,
            col: 0,
       }
    }
}

#[derive(Clone)]
pub struct Cell{
    pub image: ColorImage,
    pub is_void: bool,
    pub rank:usize,
}

impl Cell {
}

impl Default for Cell{
    fn default()-> Self{
       Self{
           image: ColorImage::example(),
           is_void: true,
           rank: 0,
       } 
    }

}



