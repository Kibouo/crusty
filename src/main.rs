mod archive;

use azul::prelude::*;

struct MyDataModel {
    current_page: usize,
    first_page: String,
    second_page: String,
    source: Vec<self::archive::ImagePage>,
    pages: Vec<String>,
    page_layout: PageLayout,
    show_help: bool,
}

impl Default for MyDataModel {
    fn default() -> Self {
        MyDataModel {
            current_page: 0,
            first_page: "".to_string(),
            second_page: "".to_string(),
            source: vec![],
            pages: vec![],
            page_layout: PageLayout::Book,
            show_help: false,
        }
    }
}

impl MyDataModel {
    fn next_page(&mut self) {
        match self.page_layout {
            PageLayout::Page => {
                if self.current_page < self.pages.len() {
                    self.current_page = self.current_page + 1;
                }
            }
            PageLayout::Book => {
                if self.current_page + 2 < self.pages.len() {
                    self.current_page = self.current_page + 2;
                }
            }
        }
    }

    fn previous_page(&mut self) {
        match self.page_layout {
            PageLayout::Page => {
                if self.current_page > 0 {
                    self.current_page = self.current_page - 1;
                }
            }
            PageLayout::Book => {
                if self.current_page != usize::min_value() {
                    self.current_page = self.current_page - 2;
                }
            }
        }
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

enum PageLayout {
    Page,
    Book,
}

impl PageLayout {
    fn toggle(&self) -> PageLayout {
        match *self {
            PageLayout::Book => PageLayout::Page,
            PageLayout::Page => PageLayout::Book,
        }
    }
}

fn update_keyboard(info: CallbackInfo<MyDataModel>) -> UpdateScreen {
    let app_state = info.state;
    let keyboard = app_state.windows[info.window_id].state.get_keyboard_state();

    let data = &mut app_state.data;
    eprintln!("{:?}", keyboard);
    let mut redraw = Redraw;
    match keyboard.current_char? {
        't' => {
            if let PageLayout::Page = data.page_layout {
                data.current_page = data.current_page - (data.current_page % 2);
            }

            data.page_layout = data.page_layout.toggle();
        }
        'n' | '\u{f702}' | ' ' => {
            data.next_page();
        }
        'p' | '\u{f703}' => {
            data.previous_page();
        }
        'q' => {
            if keyboard.super_down {
                println!("Quitting …");
                std::process::exit(1);
            }

            redraw = DontRedraw;
        }
        'h' => {
            data.toggle_help();
        }
        _ => redraw = DontRedraw,
    };

    let image: &self::archive::ImagePage = data.source.get(data.current_page).unwrap();
    if !app_state.resources.has_css_image_id(&image.filename) {
        let image_id = app_state
            .resources
            .add_css_image_id(image.filename.to_owned());
        app_state.resources.add_image_source(
            image_id,
            ImageSource::File(std::path::PathBuf::from(&image.path)),
        )
    };
    data.first_page = image.filename.to_string();

    if let PageLayout::Book = data.page_layout {
        let second_image: &self::archive::ImagePage =
            data.source.get(data.current_page + 1).unwrap();
        if !app_state.resources.has_css_image_id(&second_image.filename) {
            let image_id = app_state
                .resources
                .add_css_image_id(second_image.filename.to_owned());
            app_state.resources.add_image_source(
                image_id,
                ImageSource::File(std::path::PathBuf::from(&image.path)),
            )
        };
        data.second_page = second_image.filename.to_string();
    }

    redraw
}

impl Layout for MyDataModel {
    fn layout(&self, info: LayoutInfo<Self>) -> Dom<Self> {
        let mut dom = Dom::new(NodeType::Div)
            .with_class("comic-book")
            .with_callback(
                EventFilter::Window(WindowEventFilter::VirtualKeyUp),
                update_keyboard,
            )
            .with_child(
                Dom::new(NodeType::Div)
                    .with_class("page")
                    .with_css_override(
                        "my_image",
                        CssProperty::BackgroundContent(
                            azul::css::StyleBackgroundContent::Image(CssImageId(
                                self.first_page.to_string(),
                            ))
                            .into(),
                        ),
                    ),
            );

        if self.show_help {
            dom.add_child(Dom::div().with_class("help").with_child(Dom::label(
                "Shortcuts:\n- n, left arrow: next page(s)\n- p, right arrow: previous page(s)\n- \
                 h: help screen",
            )))
        }

        if let PageLayout::Book = self.page_layout {
            dom.add_child(
                Dom::new(NodeType::Div)
                    .with_class("page")
                    .with_css_override(
                        "my_image",
                        CssProperty::BackgroundContent(
                            azul::css::StyleBackgroundContent::Image(CssImageId(
                                self.second_page.to_string(),
                            ))
                            .into(),
                        ),
                    ),
            );
        };

        match self.page_layout {
            PageLayout::Page => {
                info.window.state.title = format!("{}", self.pages.get(self.current_page).unwrap())
            }
            PageLayout::Book => {
                info.window.state.title = format!(
                    "{} - {}",
                    self.pages.get(self.current_page + 1).unwrap(),
                    self.pages.get(self.current_page).unwrap()
                )
            }
        }

        return dom;
    }
}

fn main() {
    macro_rules! CSS_PATH {
        () => {
            concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.css")
        };
    }

    std::env::set_var("WINIT_HIDPI_FACTOR", "1.0");
    let args: Vec<String> = std::env::args().collect();

    let mut archive_name: String;
    if args.len() > 1 {
        match args[1].parse() {
            Ok(file) => archive_name = file,
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(0);
            }
        }
    } else {
        eprintln!("No argument passed.");
        std::process::exit(0);
    }

    let mut data = MyDataModel::default();
    let mut images: Vec<self::archive::ImagePage> = vec![];
    println!("Archive: {}", archive_name);
    println!("Loading images …");
    let tmp_dir = self::archive::load_images(archive_name, &mut images);
    println!("Loading images ✓");

    for image in &images {
        data.pages.push(image.filename.to_string());
    }
    data.source = images;

    let mut app = App::new(data, AppConfig::default()).unwrap();
    let css = css::override_native(include_str!(CSS_PATH!())).unwrap();
    let mut window_options = WindowCreateOptions::default();
    window_options.state.title = "Crusty".into();
    let window = app.create_window(window_options, css).unwrap();

    app.run(window).unwrap();
}
