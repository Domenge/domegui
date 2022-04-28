use eframe::{
        egui::{
            self,
            Vec2, ColorImage, 
        }, 
        epi, 
};

use super::cell::{
    Cell, 
    TaquinSide, 
    TaquinCoord,
    Direction,
};

use std::{path::Path,};
use image::{GenericImageView,};
use std::collections::HashMap;
use rand::{thread_rng, Rng,};
use log::{
    debug, 
    // error, 
    info, 
    // log_enabled, 
    // Level,
};

const BACKGROUND_IMAGE_PATH: &'static str = "./image/background.png";
const VOID_CELL_CURRENT_IMAGE_PATH: &'static str = "./image/void_cell_current.png";
const VOID_CELL_WINNER_IMAGE_PATH: &'static str = "./image/void_cell_winner.png";

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    label: String,
    cells_map: HashMap<String, Cell,>,
    sides: TaquinSide,
    image: ColorImage,
    image_void_cell:ColorImage,
    image_winner:ColorImage,
    void_cell: TaquinCoord,
    side_panel_show: bool,
    scrambled: bool,
    trace: bool,
    // this how you opt-out of serialization of a member
    //#[cfg_attr(feature = "persistence", serde(skip))]
}

#[warn(unused)]
impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            scrambled: false,
            trace: false,
            cells_map: HashMap::new(),
            sides: TaquinSide::default(),
            image: egui::ColorImage::example(),
            image_void_cell: egui::ColorImage::example(),
            image_winner: egui::ColorImage::example(),
            void_cell: TaquinCoord::default(),
            side_panel_show: false,
        }
    }
}

impl TemplateApp{
    fn add_central_panel(&mut self, ctx: &egui::Context, _frame: &epi::Frame,){
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            //
            // H E A D I N G
            //ui.heading("Taquin");

            // L A B E L
            if self.trace{
                // H Y P E R L I N K
                ui.hyperlink("https://github.com/Domenge/domegui");
                // G I T H U B _ L I N K _ F I L E
                ui.add(egui::github_link_file!(
                    "https://github.com/Domenge/domegui/blob/master/",
                    "Source code."
                ));
                ui.monospace(self.label.as_str());               
            }
            
            // G R I D
            egui::Grid::new("taquin_1")
            .spacing(Vec2::new(0.0,0.0))
            .show(ui, |ui| {

                for line in 1..=self.sides.num_line{
                    for col in 1..=self.sides.num_col{

                        let texture: &egui::TextureHandle;
                        
                        let cell = self.cells_map.get(&format!("{col}_{line}")).unwrap();
                        
                        let mut texture_opt  = None;
                        texture = texture_opt.get_or_insert_with(|| {
                            if ! cell.is_void{
                                ui.ctx()
                                    .load_texture(format!("img_col{col}_line{line}"), cell.image.clone())
                            }else if self.has_won() {
                                info!("We have a winner.");
                                ui.ctx()
                                    .load_texture(format!("img_col{col}_line{line}"), self.image_winner.clone())
                            }else{
                                ui.ctx()
                                    .load_texture(format!("img_col{col}_line{line}"), self.image_void_cell.clone())
                            }
                        });

                        let img_size = 54.0 * texture.size_vec2() / texture.size_vec2().y;

                        if ui.add(egui::ImageButton::new(texture, img_size)).clicked(){
                           // the click must fall close next to the void cell and cannot be on the void cell
                            self.on_click_button(col, line);
                        }            
                    } 
                ui.end_row();
                }                
            });
            egui::warn_if_debug_build(ui);
        });
    }

    fn add_side_panel(&mut self, ctx: &egui::Context, _frame: &epi::Frame,){
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.heading("Settings...");
//            ui.add(egui::Label::new("Settings"));
           
            ui.checkbox(&mut self.trace, "Trace");

            let mut texture_opt  = None;
            let texture: &egui::TextureHandle = texture_opt.get_or_insert_with(|| {
                // Load the texture only once.
                ui.ctx().load_texture("main-image", self.image.clone())
            });

            // Show the image:
            ui.image(texture, texture.size_vec2());            
        });
    }

    fn add_top_bottom_panel(&mut self, ctx: &egui::Context, frame: &epi::Frame,){
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Scramble").clicked() {
                        self.cells_map = self.scramble();
                    }
                    if ui.button("Settings...").clicked() {
                        self.side_panel_show = !self.side_panel_show;
                    }
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });
    }

    fn on_click_button(&mut self, col:usize, line:usize){

        let (delta , direction) = match (col, line, self.void_cell.col, self.void_cell.line){
            // same col click above void_cell
            (c,l,vcc, vcl) if c == vcc && l < vcl => (vcl - l, Direction::Up),
            // same col click below void_cell
            (c,l,vcc, vcl) if c == vcc && l > vcl => (l - vcl, Direction::Down),
            // same line click before void_cell
            (c,l,vcc, vcl) if l == vcl && c < vcc => (vcc - c, Direction::Left),
            // same line click after void_cell
            (c,l,vcc, vcl) if l == vcl && c > vcc => (c - vcc, Direction::Right),
            (_,_,_,_) => (0, Direction::Dontapply),
        };

        debug!("delta {}, direction {:?}",delta, direction);

        if direction != Direction::Dontapply{
            for _ in 0 .. delta{
                // from the void_cell shifts to the cell clicked
                
                let void_cell = self.cells_map.get(&format!("{}_{}",self.void_cell.col, self.void_cell.line)).unwrap();
                let void_cell_image = void_cell.image.clone();
                let void_cell_rank = void_cell.rank;   

                // update void_cell of the app objet
                // go get the current void_cell to put it to void
                let c = self.void_cell.col;
                let l = self.void_cell.line;


                debug!("before col {}, line {}", c, l);

                let (c, l) = match (direction, c, l){
                    (Direction::Up,    c, l)  => (c, l - 1,),
                    (Direction::Down,  c, l)  => (c, l + 1,),
                    (Direction::Left,  c, l)  => (c - 1 , l,),
                    (Direction::Right, c, l)  => (c + 1, l,),
                    (_,_,_) => panic!("Invalid direction"),
                };

                debug!("after col {}, line {}", c, l);

                // Change cell clicked
                let mut cell = self.cells_map.get_mut(&format!("{}_{}", c, l)).unwrap();
                cell.is_void = true;
                let content_image = cell.image.clone();
                let content_rank = cell.rank;

                cell.image = void_cell_image.clone();
                cell.rank = void_cell_rank;

                // Change former void_cell
                let mut cell = self.cells_map.get_mut(&format!("{}_{}", self.void_cell.col, self.void_cell.line)).unwrap();
                cell.is_void = false;
                cell.image = content_image;
                cell.rank = content_rank;

                // actualize the void_cell info in the app object
                self.void_cell.col  = c;
                self.void_cell.line = l;
            
                // do a little trace displayed in the app
                self.label = format!("On click current col {col}, line {line}, void_cell col {} line {}",
                    self.void_cell.col,
                    self.void_cell.line, 
                );

           
            }
        }else{
            self.label = "".to_owned();
        }
    }


    //
    // Scramble the image, returns a new hashmap.
    //
    fn scramble(&mut self,)-> HashMap<String, Cell,>{

        let mut rng = thread_rng();
        let mut hash:HashMap<String, Cell,> = HashMap::new();
   
        let size = self.sides.num_col* self.sides.num_line;

        let mut vect_rank:Vec<usize>= Vec::new();

        // a duplicate of the hashmap of the grid is build
        // but the rank of the cells is randomized.
        // The new hashmap is returned at the end.
        // The void cell is set anew.
        for c in 1 .. self.sides.num_col + 1 {
            for l in 1 .. self.sides.num_line + 1{
                // 
                loop{
                    let random = rng.gen_range(1..= size);
                    
                    if ! vect_rank.contains(&random){
                        vect_rank.push(random);
                        for (_,v) in self.cells_map.iter(){
                            if v.rank == random{
                                let cell = Cell{
                                        image: v.image.clone(),
                                        rank: random,
                                        is_void: v.is_void,                                                
                                };
                                // don't forget to update the void_cell
                                if cell.is_void{
                                    self.void_cell.col = c;
                                    self.void_cell.line = l;
                                };
                                hash.insert(format!("{}_{}", c, l), cell,);
                                break;              
                            }
                        }
                        break;
                    }
                }    
            }
        }
        self.scrambled = true;
        hash    
    }

    //
    // if every cell is ordered correctly i.e the ranks are sorted
    // then we win.
    //
    fn has_won(&self) -> bool{
            if self.scrambled{ 
                let mut last = 0;

            for l in 1 ..= self.sides.num_line{
                for c in 1 ..= self.sides.num_col{
                    let cell = self.cells_map.get(&format!("{c}_{l}")).unwrap();
                    last += 1;
                    let rank = cell.rank;  
                    if rank != last{
                        return false
                    }
                };
            }
            true               
        } else { 
             false 
        } 
    }
}
impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Taquin game"
    }
  
    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        env_logger::init();

        let image = image::io::Reader::open(Path::new(BACKGROUND_IMAGE_PATH)).unwrap().decode().unwrap();
        let image_void_cell = image::io::Reader::open(Path::new(VOID_CELL_CURRENT_IMAGE_PATH)).unwrap().decode().unwrap();
        let image_winner = image::io::Reader::open(Path::new(VOID_CELL_WINNER_IMAGE_PATH)).unwrap().decode().unwrap();

        let image_void_cell_buffer = image_void_cell.to_rgba8();
        let image_buffer = image.to_rgba8();
        let image_winner_buffer = image_winner.to_rgba8();

        // background image
        let cell_width = image.width() as usize / self.sides.num_col;
        let cell_height = image.height() as usize / self.sides.num_line;

        self.image = egui::ColorImage::from_rgba_unmultiplied(
            [image.width() as _, image.height() as _], 
            image_buffer.as_flat_samples().as_slice(),
        );

        // current void_cell image
        let void_cell_width  = image_void_cell.width() as usize;
        let void_cell_height = image_void_cell.height() as usize;

        self.image_void_cell = egui::ColorImage::from_rgba_unmultiplied(
            [void_cell_width as _, void_cell_height as _],
            image_void_cell_buffer.as_flat_samples().as_slice(),
        );

        // winner void_cell image
        let winner_width  = image_winner.width()  as usize;
        let winner_height = image_winner.height() as usize;

         self.image_winner = egui::ColorImage::from_rgba_unmultiplied(
            [winner_width as _, winner_height as _],
            image_winner_buffer.as_flat_samples().as_slice(),
        );

        let mut n : usize = 0;
        //
        // A grid is build through a hashmap
        // the key is "<col>_<line>"
        // the value is a Cell structure.
        for line in 1 ..=  self.sides.num_line{
            for col in 1 ..= self.sides.num_col{
    
                let sub_image = image_buffer.view(
                    (col * cell_width - cell_width).try_into().unwrap(), 
                    (line * cell_height - cell_height).try_into().unwrap(), 
                    cell_width.try_into().unwrap(), 
                    cell_height.try_into().unwrap()).to_image();
                let size = [sub_image.width() as _, sub_image.height() as _];
                let pixels = sub_image.as_flat_samples();
 
                n += 1;

                let c = Cell{
                    image: egui::ColorImage::from_rgba_unmultiplied(
                        size,
                        pixels.as_slice(),

                     ),
                    is_void: (line == self.sides.num_line  && col == self.sides.num_col),
                    rank: n,
                };
                if c.is_void {
                    self.void_cell = TaquinCoord{
                        col: col, 
                        line: line,
                    }
                }
                self.cells_map.insert(format!("{}_{}", col, line), c);
            }
        }

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    #[allow(unused_variables)]
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {

        // pattern design below
        // let Self { label, 
        //     scrambled, cells_map, 
        //     sides, image, 
        //     void_cell, side_panel_show} = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        self.add_top_bottom_panel(ctx, frame);

        self.add_central_panel(ctx, frame);

        if self.side_panel_show{
            self.add_side_panel(ctx, frame);
        }


        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }

    }


}
