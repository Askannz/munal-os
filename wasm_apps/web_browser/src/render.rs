use applib::{Color, Rect, Framebuffer};
use applib::drawing::primitives::{draw_rect, blend_rect};
use applib::drawing::text::draw_str;
use applib::drawing::text::{HACK_15, Font};
use applib::input::{InputState, InputEvent, PointerState};


pub struct Webview<'a> {
    state: State<'a>,
    view_rect: Rect,
    next_node_id: u64,
}

enum State<'a> {
    Blank,
    Active {
        buffer: Framebuffer<'a>,
        layout: LayoutNode,
        link_data: Option<LinkData>,
        y_offset: i64,
    }
}

struct LinkData {
    node_id: u64,
    rect: Rect,
    url: String,
    clicked: bool,
}

const SCROLL_SPEED: u32 = 10;

impl<'a> Webview<'a> {

    pub fn new(view_rect: &Rect) -> Self {
        Self {
            state: State::Blank,
            view_rect: view_rect.clone(),
            next_node_id: 0,
        }
    }

    pub fn update(&mut self, input_state: &InputState, html_update: Option<&str>) -> bool {

        let mut redraw = false;

        if let Some(html) = html_update {

            let layout = self.parse_html_to_layout(html);

            //debug_layout(&layout);

            let Rect { w: bw, h: bh, .. } = layout.rect;
            let mut buffer = Framebuffer::new_owned(bw, bh);
    
            draw_node(&mut buffer, &layout);

            self.state = State::Active {
                buffer,
                layout,
                link_data: None,
                y_offset: 0,
            };

            redraw = true;

        }

        match &mut self.state {
            State::Blank => (),
            State::Active { layout, link_data, y_offset, .. } => {

                for event in input_state.events {
                    if let Some(InputEvent::Scroll { delta }) = event {
                        let offset = *y_offset as i64 - delta * (SCROLL_SPEED as i64);
                        *y_offset = i64::max(0, offset);
                        redraw = true;
                    }
                }

                let PointerState { mut x, mut y, ..} = input_state.pointer;
                y = y - self.view_rect.y0 + *y_offset;
                x = x - self.view_rect.x0;

                let new_link_data = get_hovered_link(x, y, layout);

                match (&new_link_data, &link_data) {
                    (Some(new), Some(old)) if new.node_id == old.node_id => (),
                    (None, None) => (),
                    _ => { redraw = true; }
                };

                *link_data = new_link_data;

                if let Some(link_data_val) = link_data {
                    link_data_val.clicked = input_state.pointer.left_clicked;
                }
            }
        }

        redraw
    }

    pub fn draw(&self, fb: &mut Framebuffer) {

        if let State::Active { buffer, y_offset, link_data, .. } = &self.state {

            let src_rect = {
                let mut r = buffer.shape_as_rect().clone();
                r.y0 += y_offset;
                r
            };

            fb.copy_from_fb(&buffer, &src_rect, &self.view_rect, false);

            if let Some(link_data) = link_data {
                let mut r = link_data.rect.clone();
                r.y0 = r.y0 + self.view_rect.y0 - y_offset;
                r.x0 = r.x0 + self.view_rect.x0;
                blend_rect(fb, &r, Color::rgba(0, 0, 255, 128));
            }
        }
    }

    pub fn check_redirect(&self) -> Option<&str> {
        if let State::Active { link_data, .. } = &self.state {
            if let Some(link_data) = link_data {
                if link_data.clicked {
                    return Some(&link_data.url)
                }
            }
        }
        None
    }

    fn parse_html_to_layout(&mut self, html: &str) -> LayoutNode {
        let tree = scraper::Html::parse_document(html).tree;
        let root = tree.root().first_child().expect("Empty HTML");
        self.parse_node(root, 0, 0).expect("Could not parse root HTML node")
    }

    fn parse_node<'b>(&mut self, node: ego_tree::NodeRef<'b, scraper::Node>, x0: i64, y0: i64) -> Option<LayoutNode> {
    
        match node.value() {

            scraper::Node::Element(element) if element.name() == "head" => None,

            scraper::Node::Element(element) => {

                let bg_color = match element.attr("bgcolor") {
                    Some(hex_str) => Some(parse_hexcolor(hex_str)),
                    _ => None
                };

                let url: Option<String> = match element.name() {
                    "a" => element.attr("href").map(|s| s.to_owned()),
                    _ => None,
                };

                let orientation = match element.name() {
                    "tr" => Orientation::Horizontal,
                    "tbody" => Orientation::Vertical,
                    "table" => Orientation::Vertical,
                    _ => Orientation::Horizontal
                };

                let mut children: Vec<LayoutNode> = Vec::new();
                let (mut child_x0, mut child_y0): (i64, i64) = (x0, y0);
                for html_child in node.children() {
                    if let Some(child_node) = self.parse_node(html_child, child_x0, child_y0) {
                        let Rect { w: child_w, h: child_h, .. } = child_node.rect;
                        match orientation {
                            Orientation::Horizontal => child_x0 += child_w as i64,
                            Orientation::Vertical => child_y0 += child_h as i64,
                        }
                        children.push(child_node);
                    }
                }

                if children.len() > 0 {
                    let rect_0 = children[0].rect.clone();
                    let container_rect = children.iter()
                        .map(|c| c.rect.clone())
                        .fold(rect_0, |acc, r| r.bounding_box(&acc));
                    Some(LayoutNode {
                        id: self.make_node_id(),
                        rect: container_rect,
                        data: NodeData::Container { children, orientation, bg_color, url }
                    })
                } else {
                    None
                }
            },

            scraper::Node::Text(text) if check_is_whitespace(&text) => None,

            scraper::Node::Text(text) => {

                //const M: Margins = Margins { left: 0, right: 0, top: 5, bottom: 5};
                const M: Margins = Margins { left: 0, right: 0, top: 0, bottom: 0};

                let text = core::str::from_utf8(text.as_bytes()).expect("Not UTF-8");
                let font = &HACK_15; // TODO
                let w = (text.len() * font.char_w) as u32 + M.left + M.right;
                let h = font.char_h as u32  + M.top + M.bottom;

                Some(LayoutNode {
                    id: self.make_node_id(),
                    rect: Rect { 
                        x0: x0 + M.left as i64,
                        y0: y0 + M.top as i64,
                        w, h
                    },
                    data: NodeData::Text { 
                        text: text.to_string(),
                        color: Color::BLACK,  // TODO
                        font, 
                        url: None,
                    }
                })
            },

            _ => None
        }
    }

    fn make_node_id(&mut self) -> u64 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }
}

fn draw_node(fb: &mut Framebuffer, node: &LayoutNode) {

    let rect = &node.rect;

    if fb.w as i64 <= rect.x0 || fb.h as i64 <= rect.y0 {
        return;
    }

    match &node.data {
        NodeData::Text { text, color, font, .. } => {
            draw_str(fb, text, rect.x0, rect.y0, font, *color, None);
        },
        NodeData::Container { children, bg_color, .. } => {

            if let &Some(bg_color) = bg_color {
                draw_rect(fb, &rect, bg_color);
            }

            for child in children.iter() {
                draw_node(fb, child);
            }
        }
    }
}

fn get_hovered_link(x: i64, y: i64, node: &LayoutNode) -> Option<LinkData> {

    let rect = &node.rect;

    match &node.data {
        NodeData::Text { .. } => None,
        NodeData::Container { children, url, .. } => match rect.check_contains_point(x, y) {
            true => match url {
                Some(url) => Some(LinkData {
                    node_id: node.id,
                    rect: rect.clone(),
                    url: url.clone(),
                    clicked: false,
                }),
                None => children.iter().find_map(|c| get_hovered_link(x, y, c))
            },
            false => None
        }
    }

}

fn debug_layout(root_node: &LayoutNode) {

    fn repr_node(out_str: &mut String, node: &LayoutNode, depth: usize) {

        match &node.data {
            NodeData::Text { text, .. } => {
                out_str.push_str(&format!("{}{}\n"," ".repeat(depth), text));
            },
            NodeData::Container { children, orientation, .. } => {
                out_str.push_str(&format!("{}CONTAINER {:?}\n"," ".repeat(depth), orientation));
                for child in children {
                    repr_node(out_str, child, depth+1);
                }
            }
        }
    }

    let mut out_str = String::new();
    repr_node(&mut out_str, root_node, 0);

    guestlib::print_console(&out_str);

}

enum NodeData {
    Text { text: String, color: Color, font: &'static Font, url: Option<String> },
    Container { children: Vec<LayoutNode>, orientation: Orientation, bg_color: Option<Color>, url: Option<String> }
}

struct LayoutNode {
    id: u64,
    rect: Rect,
    data: NodeData,
}

#[derive(Debug, Clone, Copy)]
enum Orientation {
    Horizontal,
    Vertical,
}

struct Margins {
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
}

fn check_is_whitespace(s: &str) -> bool {
    s.chars().map(|c| char::is_whitespace(c)).all(|x| x)
}

fn parse_hexcolor(hex_str: &str) -> Color {

    let mut color_bytes = hex::decode(hex_str.replace("#", "")).expect("Invalid color");

    match color_bytes.len() {
        3 => color_bytes.push(255),
        4 => (),
        _ => panic!("Invalid color: {:?}", color_bytes)
    };

    let color_bytes: [u8; 4] = color_bytes.try_into().unwrap();

    Color::from_u32(u32::from_le_bytes(color_bytes))
}