#[derive(Debug)]
pub struct Config {
    pub theme: iced::Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: iced::Theme::TokyoNight,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
