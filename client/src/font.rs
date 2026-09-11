use std::borrow::Cow;

pub fn load() -> Vec<Cow<'static, [u8]>> {
    vec![
        include_bytes!("../assets/fonts/icons.ttf")
            .as_slice()
            .into(),
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")
            .as_slice()
            .into(),
    ]
}
