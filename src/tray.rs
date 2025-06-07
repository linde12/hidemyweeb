use image::ImageReader;
use {std::io::Cursor, tray_item::IconSource, tray_item::TrayItem};

fn get_icon_buffer(icon: &[u8]) -> Vec<u8> {
    let img = ImageReader::new(Cursor::new(icon))
        .with_guessed_format()
        .expect("Failed to guess image format")
        .decode()
        .expect("Failed to decode image");

    let mut pixels = img.into_rgba8().into_vec();

    // Convert RGBA to ARGB to get correct colors
    for pixel in pixels.chunks_exact_mut(4) {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        pixel[0] = a;
        pixel[1] = r;
        pixel[2] = g;
        pixel[3] = b;
    }
    pixels
}

pub enum IconType {
    Recording,
    Idle,
}

pub struct Tray {
    tray: TrayItem,
    recording_icon: Vec<u8>,
    idle_icon: Vec<u8>,
}
impl Tray {
    pub fn new() -> Self {
        let recording_icon = get_icon_buffer(include_bytes!("../resources/recording.png"));
        let idle_icon = get_icon_buffer(include_bytes!("../resources/idle.png"));
        let mut tray = Tray {
            tray: TrayItem::new(
                "HideMyWeeb",
                IconSource::Data {
                    data: idle_icon.clone(),
                    height: 2048,
                    width: 2048,
                },
            )
            .unwrap(),
            recording_icon,
            idle_icon,
        };

        tray.tray.add_label("HideMyWeeb").unwrap();

        tray
    }

    pub fn set_icon(&mut self, icon_type: IconType) {
        let icon = match icon_type {
            IconType::Recording => &self.recording_icon,
            IconType::Idle => &self.idle_icon,
        };
        self.tray
            .set_icon(IconSource::Data {
                height: 2048,
                width: 2048,
                data: icon.clone(),
            })
            .unwrap();
    }
}
