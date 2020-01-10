// Example 3: The RGB Triangle
// Open a window, and draw the standard GPU triangle
use quicksilver::{geom::Vector, graphics::{Color, Element, Graphics, Mesh, Vertex}, lifecycle::{run, EventStream, Settings, Window}, Result, load_file};
use rusttype::{Font as RTFont, Scale, point};
use quicksilver::geom::Rectangle;
use quicksilver::graphics::{Image, PixelFormat};
use crate::glyph::Glyph;
use pathfinding::directed::dijkstra::dijkstra;

fn main() {
    run(
        Settings {
            size: Vector::new(800.0, 600.0).into(),
            title: "RGB Triangle Example",
            ..Settings::default()
        },
        app,
    );
}


pub mod font {
    use std::path::Path;
    use rusttype::{Font as RTFont, Scale, point, PositionedGlyph};
    use quicksilver::{Result, load_file};
    use quicksilver::graphics::{Graphics, Image, PixelFormat};

    pub struct Font {
        pub(crate) data: RTFont<'static>
    }

    impl Font {
        pub async fn load(path: impl AsRef<Path>) -> Result<Font> {
            let file_contents = load_file(path).await?;
            let font = RTFont::from_bytes(file_contents).unwrap();
            Ok(
                Font {
                    data: font
                }
            )
        }

        pub fn render(&self, gfx: &Graphics, text: &str, size: f32) -> Result<Image> {
            // Most of this is either from, or inspired by, quicksilver
            let scale = Scale::uniform(size);
            let offset = point(0.0, self.data.v_metrics(scale).ascent);
            let line_count = text.lines().count();
            let glyphs_per_line = text
                .lines()
                .map(|text| {
                    //Avoid clipping
                    let offset = point(0.0, self.data.v_metrics(scale).ascent);
                    let glyphs = self.data.layout(text.trim(), scale, offset)
                        .collect::<Vec<PositionedGlyph>>();
                    let width = glyphs.iter().rev()
                        .map(|g|
                            g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
                        .next().unwrap_or(0.0).ceil() as usize;
                    (glyphs, width)
                })
                .collect::<Vec<_>>();
            let base_width = *glyphs_per_line.iter().map(|(_, width)| width).max().unwrap_or(&0);
            let max_width = round_pow_2(base_width as i32) as usize;
            let height = round_pow_2((size * (line_count as f32)).ceil() as i32) as usize;
            let mut imgbuf = image::ImageBuffer::new(max_width as u32, height as u32);
            for (line_index, (glyphs, width)) in glyphs_per_line.iter().enumerate() {
                for glyph in glyphs {
                    if let Some(bounding_box) = glyph.pixel_bounding_box() {
                        glyph.draw(|_x, _y, v| {
                            let x = _x + std::cmp::max(0, bounding_box.min.x) as u32;
                            let y = _y + std::cmp::max(0, bounding_box.min.y) as u32;
                            if x < *width as u32 && y < size as u32 {
                                let pixel = imgbuf.get_pixel_mut(x, y + (line_index * size as usize) as u32);
                                *pixel = image::Rgba([255, 255, 255, (255.0 * v) as u8]);
                            }
                        });
                    }
                }
            }

            imgbuf.save("sv.png");
            Image::from_raw(gfx,
                            &imgbuf.into_raw(),
                            max_width as u32,
                            height as u32,
                            PixelFormat::RGBA)
        }
    }

    fn round_pow_2(n: i32) -> i32 {
        (2.0 as f32).powi(((n as f32).log2() + 1.0) as i32) as i32
    }
}

pub mod tileset {
    use std::collections::HashMap;
    use quicksilver::{
        graphics::{Image, Graphics},
        geom::{Rectangle, Vector},
        Result
    };
    use crate::font::Font;
    use crate::glyph::Glyph;

    pub struct Tileset {
        image: Image,
        map: HashMap<char, Rectangle>,
    }

    impl Tileset {
        pub async fn from_font(gfx: &Graphics, path: &str) -> Result<Tileset> {
            let lines = r#"╦╩═╬╧╨╤╥╙╘╒╓╫╪┘╠┌█▄▌▐▀αßΓπΣσµτΦδ∞φ╟╚╔║╗╝╣╢╖
*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQ⌠⌡≥
RSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxy÷≈
z{|} ~⌂ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáí°∙
óúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╕╜╛┐└┴┬├─┼╞·√±≤ⁿε∩≡ΘΩ
"☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼!#$%&'()²■"#;
            let font = Font::load(path).await.expect("failed to load font");
            let size = 40.0;
            let font_image = font.render(&gfx, &lines, size).expect("failed to load font image");
            let tile_size_px = Vector::new(size, size);
            let mut map = HashMap::new();
            for (line_num, glyphs) in lines.lines().enumerate() {
                for (index, glyph) in glyphs.chars().enumerate() {
                    let pos = (index as i32 * tile_size_px.x as i32, (line_num * tile_size_px.y as usize) as i32);
                    let rect = Rectangle::new(pos, tile_size_px);
                    map.insert(glyph, rect);
                }
            }
            Ok(
                Tileset {
                    image: font_image,
                    map,
                }
            )
        }
        pub fn draw(&self, gfx: &mut Graphics, glyph: &Glyph, pos: Rectangle) {
            let image = &self.image;
            let rect = self.map.get(&glyph.ch).unwrap();
            if let Some(background) = &glyph.background {
                gfx.fill_rect(&pos, *background);
            }
            if let Some(foreground) = &glyph.foreground {
                gfx.draw_subimage_tinted(image, *rect, pos, *foreground);
            } else {
                gfx.draw_subimage(image, *rect, pos);
            }
        }
        pub fn draw_char(&self, gfx: &mut Graphics, glyph: char, pos: Rectangle) {
            let image = &self.image;
            let rect = self.map.get(&glyph).unwrap();
            println!("{:?}", rect);
            let r = Rectangle::new((rect.pos.x + 1.0, rect.pos.y + 1.0), (rect.size.x - 2.0, rect.size.y - 2.0));
            gfx.draw_subimage(image, r, pos);
        }
    }
}

pub mod glyph {
    use quicksilver::graphics::Color;
    #[derive(Debug)]
    pub struct Glyph {
        pub ch: char,
        pub foreground: Option<Color>,
        pub background: Option<Color>,
    }
}

pub mod grid {
    use quicksilver::{
        geom::{Rectangle, Vector}
    };

    pub struct Grid {
        width_multi: u32,
        height_multi: u32
    }

    impl Grid {
        pub fn from_screen_size(grid_size: impl Into<Vector>, screen_size: impl Into<Vector>) -> Self {
            let grid = grid_size.into();
            let screen = screen_size.into();
            Grid {
                width_multi: (screen.x / grid.x) as u32,
                height_multi: (screen.y / grid.y) as u32,
            }
        }

        pub fn from_tile_size(tile_size: impl Into<Vector>) -> Self {
            let tile = tile_size.into();
            Grid {
                width_multi: tile.x as u32,
                height_multi: tile.y as u32,
            }
        }

        pub fn rect(&self, x: u32, y: u32) -> Rectangle {
            Rectangle::new(
                (self.width_multi * x, self.height_multi * y),
                (self.width_multi, self.height_multi)
            )
        }
    }
}

pub fn get_char_from_edges(edges: i32) -> char {
    match edges {
        0b1101 => '╩',
        0b1001 => '╚',
        0b1010 => '╔',
        0b1100 => '═',
        0b1000 => '═',
        0b0100 => '═',
        0b0011 => '║',
        0b0010 => '║',
        0b0001 => '║',
        0b0110 => '╗',
        0b0101 => '╝',
        0b0111 => '╣',
        0b1111 => '╬',
        0b1110 => '╦',
        0b1011 => '╠',
        0b0000 => ' ',
        _ => 'A'
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pos(usize, usize);

pub fn visitable_tiles(p: &Pos, maze: &Vec<Vec<char>>) -> Vec<(Pos, usize)> {
    let width = maze[0].len();
    let height = maze.len();
    let mut succ = vec![];
    if p.1 < height - 1 && maze[p.1 + 1][p.0] == ' ' {
        succ.push((Pos(p.0, p.1 + 1), 1))
    }
    if p.1 > 0 && maze[p.1 - 1][p.0] == ' ' {
        succ.push((Pos(p.0, p.1 - 1), 1))
    }
    if p.0 < width - 1 && maze[p.1][p.0 + 1] == ' ' {
        succ.push((Pos(p.0 + 1, p.1), 1))
    }
    if p.0 > 0 && maze[p.1][p.0 - 1] == ' ' {
        succ.push((Pos(p.0 - 1, p.1), 1))
    }
    println!("{:?}", succ);
    succ
}

pub fn maze_to_vec(maze: &str) -> (Pos, Pos, Vec<Vec<char>>) {
    let mut start = Pos(0, 0);
    let mut end = Pos(0, 0);
    let mut conv: Vec<Vec<char>> = vec![];
    for (y, line) in maze.lines().enumerate() {
        let mut v: Vec<char> = vec![];
        for (x, c) in line.chars().enumerate() {
            if c == '☻' {
                start = Pos(x, y);
            }
            if c == '♥' {
                end = Pos(x, y);
            }
            v.push(c);
        }
        conv.push(v);
    }
    (start, end, conv)
}

pub fn maze_renderer(conv: &Vec<Vec<char>>) -> Vec<(Glyph, Vector)> {
    let mut res = vec![];
    let height = conv.len();
    let width = conv[0].len();

    for x in 0..width {
        for y in 0..height {
            let mut edges = 0b0;
            if conv[y][x] != '#' {

                continue;
            }
            if x > 0 && conv[y][x - 1] == '#' {
                edges = edges | 0b0100;
            }
            if x < width - 1 && conv[y][x + 1] == '#' {
                edges = edges | 0b1000;
            }
            if y > 0 && conv[y - 1][x] == '#' {
                edges = edges | 0b0001;
            }
            if y < height - 1 && conv[y + 1][x] == '#' {
                edges = edges | 0b0010;
            }
            let ch = get_char_from_edges(edges);
            let glyph = Glyph {
                ch,
                foreground: Some(Color::BLACK),
                background: None
            };
            res.push((glyph, (x as u32, y as u32).into()));
        }
    }
    res
}

async fn app(window: Window, mut gfx: Graphics, mut events: EventStream) -> Result<()> {
    let maze = "\
##################################################################### ♥ #
#   #               #               #           #                   #   #
#   #   #########   #   #####   #########   #####   #####   #####   #   #
#               #       #   #           #           #   #   #       #   #
#########   #   #########   #########   #####   #   #   #   #########   #
#       #   #               #           #   #   #   #   #           #   #
#   #   #############   #   #   #########   #####   #   #########   #   #
#   #               #   #   #       #           #           #       #   #
#   #############   #####   #####   #   #####   #########   #   #####   #
#           #       #   #       #   #       #           #   #           #
#   #####   #####   #   #####   #   #########   #   #   #   #############
#       #       #   #   #       #       #       #   #   #       #       #
#############   #   #   #   #########   #   #####   #   #####   #####   #
#           #   #           #       #   #       #   #       #           #
#   #####   #   #########   #####   #   #####   #####   #############   #
#   #       #           #           #       #   #   #               #   #
#   #   #########   #   #####   #########   #   #   #############   #   #
#   #           #   #   #   #   #           #               #   #       #
#   #########   #   #   #   #####   #########   #########   #   #########
#   #       #   #   #           #           #   #       #               #
#   #   #####   #####   #####   #########   #####   #   #########   #   #
#   #                   #           #               #               #   #
# ☻ #####################################################################\
";
    let (start, end, maze_vec) = maze_to_vec(maze);

    let maze = maze_renderer(&maze_vec);

    let tileset = tileset::Tileset::from_font(&gfx, "Px437_PhoenixEGA_8x8.ttf").await?;
    gfx.clear(Color::WHITE);
    // let rect = Rectangle::new(Vector::new(350.0, 100.0), Vector::new(100.0, 100.0));
    // gfx.fill_rect(&rect, Color::RED);
    let grid = grid::Grid::from_tile_size((10.0, 10.0));
    for (glyph, pos) in maze {
        tileset.draw(&mut gfx, &glyph, grid.rect(pos.x as u32 + 2, pos.y as u32 + 2));
    }
    let result = dijkstra(&start, |p| visitable_tiles(p, &maze_vec), |p| *p == Pos(71, 1));
    let mut path: Vec<(Glyph, Pos)> = vec![];
    if let Some((p, _)) = result {
        for pos in p {
            let glyph = Glyph {
                    ch: '▒',
                    foreground: Some(Color::GREEN),
                    background: None
                };
            tileset.draw(&mut gfx, &glyph, grid.rect(pos.0 as u32 + 2, pos.1 as u32 + 2));
        }
    }

    gfx.present(&window)?;
    loop {
        while let Some(_) = events.next_event().await {}
    }
}