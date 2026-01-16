pub struct Config {
    theme: iced::Theme,
}

pub fn build_config() -> Config {
    Config {
        theme: iced::Theme::TokyoNight,
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
