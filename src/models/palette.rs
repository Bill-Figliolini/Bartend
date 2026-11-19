//! # Palette
//! User-defined palettes: a name and the eight colors the app is drawn
//! with, and the file they are kept in. Kept free of iced; presentation
//! turns them into themes.

use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{self, OpenOptions},
    io::{BufWriter, ErrorKind},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::models::ConfigError;

/// A color as 8-bit red, green and blue. Saved as `"#RRGGBB"`, so a
/// config file stays readable and can be edited by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Text that is not six hex digits, with or without a leading `#`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbParseError(pub String);

impl Display for RgbParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "'{}' is not a color; use six hex digits like #D8C08A",
            self.0
        )
    }
}

impl std::error::Error for RgbParseError {}

impl FromStr for Rgb {
    type Err = RgbParseError;

    /// Reads six hex digits, in either case, with or without a leading
    /// `#`. Surrounding whitespace is ignored.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let trimmed = text.trim();
        let digits = trimmed.strip_prefix('#').unwrap_or(trimmed);
        // Checked first: `from_str_radix` would accept a leading `+`, and
        // slicing by byte is only safe once every character is ASCII.
        if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(RgbParseError(text.to_string()));
        }
        let channel = |at: usize| {
            u8::from_str_radix(&digits[at..at + 2], 16).expect("two hex digits always fit in a u8")
        };
        Ok(Self::new(channel(0), channel(2), channel(4)))
    }
}

impl Display for Rgb {
    /// Writes `#RRGGBB` in upper case.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl TryFrom<String> for Rgb {
    type Error = RgbParseError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl From<Rgb> for String {
    fn from(color: Rgb) -> Self {
        color.to_string()
    }
}

/// The eight colors a palette is made of, by the part of the app each
/// one paints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteColors {
    /// The panels behind everything.
    pub background: Rgb,
    /// The frame, cards and tiles.
    pub surface: Rgb,
    /// The lines edging cards and the frame.
    pub hairline: Rgb,
    /// The accent, and the fill of primary buttons.
    pub accent: Rgb,
    pub text: Rgb,
    pub success: Rgb,
    pub warning: Rgb,
    pub danger: Rgb,
}

/// A palette the user made, saved in the config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomPalette {
    /// Shown in the palette picker; see [`palette_name`] for the rules.
    pub name: String,
    pub colors: PaletteColors,
}

/// Why a palette name was turned down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteNameError {
    /// The name is empty or only whitespace.
    Empty,
    /// Another palette already goes by this name, ignoring case. Holds
    /// that palette's name as it is spelled.
    Taken(String),
}

impl Display for PaletteNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaletteNameError::Empty => write!(f, "A palette needs a name"),
            PaletteNameError::Taken(name) => {
                write!(f, "There is already a palette called '{name}'")
            }
        }
    }
}

impl std::error::Error for PaletteNameError {}

/// Checks `name` as the name of a palette, giving it back trimmed.
///
/// `taken` is every other palette's name, built-in and custom. When
/// renaming a palette, leave its current name out, so it can keep that
/// name or change only its case.
///
/// # Errors
///
/// [`PaletteNameError::Empty`] if `name` is blank, or
/// [`PaletteNameError::Taken`] if it matches a name in `taken`, ignoring
/// case and surrounding whitespace.
pub fn palette_name<'a>(
    name: &str,
    taken: impl IntoIterator<Item = &'a str>,
) -> Result<String, PaletteNameError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(PaletteNameError::Empty);
    }
    let folded = name.to_lowercase();
    match taken
        .into_iter()
        .find(|other| other.trim().to_lowercase() == folded)
    {
        Some(other) => Err(PaletteNameError::Taken(other.trim().to_string())),
        None => Ok(name.to_string()),
    }
}

/// The palettes the user made, kept in their own file beside the config
/// (see [`Config::palettes_path`](crate::models::Config::palettes_path)).
///
/// Read only where the palettes are worked with, in Settings, rather than
/// traveling with the config; the config keeps its own copy of whichever
/// palette is in use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteFile {
    path: PathBuf,
    palettes: Vec<CustomPalette>,
    /// The names of palettes in the file that could not be read when it
    /// was loaded. They are left out of `palettes`, so the next save drops
    /// them from the file.
    skipped: Vec<String>,
}

impl PaletteFile {
    /// Reads the palettes saved at `path`; no file means no palettes yet.
    ///
    /// Each palette is read on its own, so one bad palette doesn't lose the
    /// rest. A palette is skipped if it is malformed, has a bad or missing
    /// color, or its name is blank or (ignoring case) repeats an earlier
    /// palette's. The names kept are trimmed. A skipped palette without a
    /// usable name is called by its place in the list, such as
    /// `palette 2`.
    ///
    /// # Errors
    ///
    /// [`ConfigError::UnreadablePalettes`] if the file exists but cannot
    /// be read, or is not a JSON list.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => String::from("[]"),
            Err(_) => return Err(ConfigError::UnreadablePalettes),
        };
        let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&text) else {
            return Err(ConfigError::UnreadablePalettes);
        };
        let mut palettes: Vec<CustomPalette> = Vec::new();
        let mut skipped = Vec::new();
        for (index, entry) in entries.into_iter().enumerate() {
            let label = entry
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map_or_else(|| format!("palette {}", index + 1), str::to_string);
            let read = serde_json::from_value::<CustomPalette>(entry)
                .ok()
                .and_then(|palette| {
                    let taken = palettes.iter().map(|kept| kept.name.as_str());
                    let name = palette_name(&palette.name, taken).ok()?;
                    Some(CustomPalette { name, ..palette })
                });
            match read {
                Some(palette) => palettes.push(palette),
                None => skipped.push(label),
            }
        }
        Ok(Self {
            path,
            palettes,
            skipped,
        })
    }

    /// Writes the palettes to the file, replacing what was there.
    ///
    /// # Errors
    ///
    /// [`ConfigError::UnableToSavePalettes`] if the file can't be written.
    pub fn save(&self) -> Result<(), ConfigError> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|_| ConfigError::UnableToSavePalettes)?;
        serde_json::to_writer(BufWriter::new(file), &self.palettes)
            .map_err(|_| ConfigError::UnableToSavePalettes)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Puts `palette` in the list: in place of the palette called
    /// `replacing` if there is one, so an edit keeps its place even when
    /// renamed, or else at the end. Names aren't checked here; see
    /// [`palette_name`].
    pub fn put(&mut self, replacing: Option<&str>, palette: CustomPalette) {
        let slot = replacing.and_then(|name| self.palettes.iter_mut().find(|p| p.name == name));
        match slot {
            Some(slot) => *slot = palette,
            None => self.palettes.push(palette),
        }
    }

    /// Takes the palette called `name` out of the list, saying whether
    /// there was one.
    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.palettes.len();
        self.palettes.retain(|palette| palette.name != name);
        self.palettes.len() != before
    }

    /// The palettes, in the order they were made.
    #[must_use]
    pub fn palettes(&self) -> &[CustomPalette] {
        &self.palettes
    }

    /// The names of the palettes that could not be read; see
    /// [`load`](Self::load).
    #[must_use]
    pub fn skipped(&self) -> &[String] {
        &self.skipped
    }
}

/// A palette called `name` in the Navy & Champagne colors, for tests.
#[cfg(test)]
pub(crate) fn sample(name: &str) -> CustomPalette {
    CustomPalette {
        name: name.to_string(),
        colors: PaletteColors {
            background: Rgb::new(0x0F, 0x16, 0x26),
            surface: Rgb::new(0x17, 0x20, 0x36),
            hairline: Rgb::new(0x2D, 0x3A, 0x57),
            accent: Rgb::new(0xD8, 0xC0, 0x8A),
            text: Rgb::new(0xF0, 0xEB, 0xDD),
            success: Rgb::new(0x86, 0xB3, 0x9A),
            warning: Rgb::new(0xE2, 0xB6, 0x5C),
            danger: Rgb::new(0xC4, 0x6A, 0x6A),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod rgb {
        use super::*;

        #[test]
        fn reads_six_hex_digits_with_or_without_a_hash() {
            let champagne = Rgb::new(0xD8, 0xC0, 0x8A);
            assert_eq!("#D8C08A".parse(), Ok(champagne));
            assert_eq!("D8C08A".parse(), Ok(champagne));
        }

        #[test]
        fn reads_either_case_and_ignores_surrounding_whitespace() {
            assert_eq!(" #d8c08A\t".parse(), Ok(Rgb::new(0xD8, 0xC0, 0x8A)));
        }

        #[test]
        fn reads_the_extremes() {
            assert_eq!("#000000".parse(), Ok(Rgb::new(0, 0, 0)));
            assert_eq!("#FFFFFF".parse(), Ok(Rgb::new(255, 255, 255)));
        }

        #[test]
        fn rejects_anything_but_six_hex_digits() {
            for bad in [
                "", "#", "#FFF", "#FFFFF", "#FFFFFFF", "#GG0000", "##FFFFFF", "+F+F+F", "#FF 00 0",
                "#ÿÿÿ",
            ] {
                assert_eq!(
                    bad.parse::<Rgb>(),
                    Err(RgbParseError(bad.to_string())),
                    "{bad:?}"
                );
            }
        }

        #[test]
        fn writes_upper_case_with_a_hash() {
            assert_eq!(Rgb::new(0x0F, 0x16, 0x26).to_string(), "#0F1626");
        }

        #[test]
        fn round_trips_through_text() {
            for color in [
                Rgb::new(0, 0, 0),
                Rgb::new(255, 255, 255),
                Rgb::new(0x0F, 0xA0, 0x7B),
            ] {
                assert_eq!(color.to_string().parse(), Ok(color));
            }
        }

        #[test]
        fn is_saved_as_a_hex_string() {
            let json = serde_json::to_value(Rgb::new(0xD8, 0xC0, 0x8A)).unwrap();
            assert_eq!(json, serde_json::json!("#D8C08A"));
        }

        #[test]
        fn loads_from_a_hex_string() {
            let color: Rgb = serde_json::from_value(serde_json::json!("#d8c08a")).unwrap();
            assert_eq!(color, Rgb::new(0xD8, 0xC0, 0x8A));
        }

        #[test]
        fn will_not_load_from_anything_else() {
            serde_json::from_value::<Rgb>(serde_json::json!("#nope!!")).unwrap_err();
            serde_json::from_value::<Rgb>(serde_json::json!(14_205_066)).unwrap_err();
        }

        #[test]
        fn a_parse_error_names_the_text_and_the_format() {
            assert_eq!(
                RgbParseError("teal".to_string()).to_string(),
                "'teal' is not a color; use six hex digits like #D8C08A"
            );
        }
    }

    mod custom_palette {
        use super::*;

        #[test]
        fn is_saved_as_a_name_and_named_hex_colors() {
            let json = serde_json::to_value(sample("Harbor")).unwrap();
            assert_eq!(
                json,
                serde_json::json!({
                    "name": "Harbor",
                    "colors": {
                        "background": "#0F1626",
                        "surface": "#172036",
                        "hairline": "#2D3A57",
                        "accent": "#D8C08A",
                        "text": "#F0EBDD",
                        "success": "#86B39A",
                        "warning": "#E2B65C",
                        "danger": "#C46A6A",
                    }
                })
            );
        }

        #[test]
        fn round_trips_through_json() {
            let palette = sample("Harbor");
            let json = serde_json::to_string(&palette).unwrap();
            assert_eq!(
                serde_json::from_str::<CustomPalette>(&json).unwrap(),
                palette
            );
        }

        #[test]
        fn will_not_load_with_a_bad_or_missing_color() {
            let mut json = serde_json::to_value(sample("Harbor")).unwrap();
            json["colors"]["accent"] = serde_json::json!("gold");
            serde_json::from_value::<CustomPalette>(json.clone()).unwrap_err();

            json["colors"].as_object_mut().unwrap().remove("accent");
            serde_json::from_value::<CustomPalette>(json).unwrap_err();
        }
    }

    mod palette_file {
        use super::*;
        use tempfile::{TempDir, tempdir};

        /// Writes `json` as the palettes file in `dir`, and loads it.
        fn load_json(dir: &TempDir, json: &serde_json::Value) -> PaletteFile {
            let path = dir.path().join("palettes.json");
            fs::write(&path, json.to_string()).unwrap();
            PaletteFile::load(path).unwrap()
        }

        /// `sample(name)` as JSON.
        fn palette_json(name: &str) -> serde_json::Value {
            serde_json::to_value(sample(name)).unwrap()
        }

        #[test]
        fn no_file_means_no_palettes() {
            let dir = tempdir().unwrap();
            let file = PaletteFile::load(dir.path().join("none.json")).unwrap();
            assert_eq!(file.palettes(), []);
            assert_eq!(file.skipped(), Vec::<String>::new());
            assert!(!file.path().exists());
        }

        #[test]
        fn saves_and_loads_palettes_in_order() {
            let dir = tempdir().unwrap();
            let path = dir.path().join("palettes.json");
            let file = PaletteFile {
                path: path.clone(),
                palettes: vec![sample("Harbor"), sample("Dusk")],
                skipped: Vec::new(),
            };

            file.save().unwrap();

            assert_eq!(PaletteFile::load(&path).unwrap(), file);
        }

        #[test]
        fn is_saved_as_a_list_of_palettes() {
            let dir = tempdir().unwrap();
            let path = dir.path().join("palettes.json");
            PaletteFile {
                path: path.clone(),
                palettes: vec![sample("Harbor")],
                skipped: Vec::new(),
            }
            .save()
            .unwrap();

            let json: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
            assert_eq!(json, serde_json::json!([palette_json("Harbor")]));
        }

        #[test]
        fn skips_only_the_palettes_it_cannot_read() {
            let mut bad_color = palette_json("Bad Color");
            bad_color["colors"]["text"] = serde_json::json!("#12345");
            let mut missing_color = palette_json("Missing Color");
            missing_color["colors"]
                .as_object_mut()
                .unwrap()
                .remove("danger");
            let dir = tempdir().unwrap();

            let file = load_json(
                &dir,
                &serde_json::json!([
                    palette_json("Harbor"),
                    bad_color,
                    missing_color,
                    palette_json("Dusk"),
                ]),
            );

            assert_eq!(file.palettes(), [sample("Harbor"), sample("Dusk")]);
            assert_eq!(file.skipped(), ["Bad Color", "Missing Color"]);
        }

        #[test]
        fn skips_a_palette_whose_name_repeats_an_earlier_one() {
            let dir = tempdir().unwrap();
            let file = load_json(
                &dir,
                &serde_json::json!([palette_json("Harbor"), palette_json(" HARBOR ")]),
            );

            assert_eq!(file.palettes(), [sample("Harbor")]);
            assert_eq!(file.skipped(), ["HARBOR"]);
        }

        #[test]
        fn trims_the_names_it_keeps() {
            let dir = tempdir().unwrap();
            let file = load_json(&dir, &serde_json::json!([palette_json("  Harbor  ")]));

            assert_eq!(file.palettes(), [sample("Harbor")]);
        }

        #[test]
        fn calls_a_skipped_palette_without_a_usable_name_by_its_place() {
            let mut blank = palette_json("Blank");
            blank["name"] = serde_json::json!("   ");
            let mut unnamed = palette_json("Unnamed");
            unnamed.as_object_mut().unwrap().remove("name");
            let dir = tempdir().unwrap();

            let file = load_json(
                &dir,
                &serde_json::json!([
                    palette_json("Harbor"),
                    blank,
                    serde_json::json!("not a palette"),
                    unnamed,
                ]),
            );

            assert_eq!(file.palettes(), [sample("Harbor")]);
            assert_eq!(file.skipped(), ["palette 2", "palette 3", "palette 4"]);
        }

        #[test]
        fn the_next_save_drops_skipped_palettes() {
            let mut bad = palette_json("Bad");
            bad["colors"]["accent"] = serde_json::json!("gold");
            let dir = tempdir().unwrap();
            let file = load_json(&dir, &serde_json::json!([palette_json("Harbor"), bad]));

            file.save().unwrap();
            let reloaded = PaletteFile::load(file.path()).unwrap();

            assert_eq!(reloaded.palettes(), [sample("Harbor")]);
            assert_eq!(reloaded.skipped(), Vec::<String>::new());
        }

        /// A file at `dir` holding `names`, as samples.
        fn file_of(dir: &TempDir, names: &[&str]) -> PaletteFile {
            PaletteFile {
                path: dir.path().join("palettes.json"),
                palettes: names.iter().map(|name| sample(name)).collect(),
                skipped: Vec::new(),
            }
        }

        fn names(file: &PaletteFile) -> Vec<&str> {
            file.palettes()
                .iter()
                .map(|palette| palette.name.as_str())
                .collect()
        }

        #[test]
        fn put_adds_a_new_palette_at_the_end() {
            let dir = tempdir().unwrap();
            let mut file = file_of(&dir, &["Harbor", "Dusk"]);
            file.put(None, sample("Dawn"));
            assert_eq!(names(&file), ["Harbor", "Dusk", "Dawn"]);
        }

        #[test]
        fn put_replaces_an_edited_palette_in_its_place_even_when_renamed() {
            let dir = tempdir().unwrap();
            let mut file = file_of(&dir, &["Harbor", "Dusk", "Dawn"]);
            let mut renamed = sample("Midnight");
            renamed.colors.accent = Rgb::new(1, 2, 3);

            file.put(Some("Dusk"), renamed.clone());

            assert_eq!(names(&file), ["Harbor", "Midnight", "Dawn"]);
            assert_eq!(file.palettes()[1], renamed);
        }

        #[test]
        fn put_adds_at_the_end_when_the_edited_palette_is_gone() {
            let dir = tempdir().unwrap();
            let mut file = file_of(&dir, &["Harbor"]);
            file.put(Some("Gone"), sample("Dusk"));
            assert_eq!(names(&file), ["Harbor", "Dusk"]);
        }

        #[test]
        fn remove_takes_out_only_the_named_palette() {
            let dir = tempdir().unwrap();
            let mut file = file_of(&dir, &["Harbor", "Dusk", "Dawn"]);
            assert!(file.remove("Dusk"));
            assert_eq!(names(&file), ["Harbor", "Dawn"]);
            assert!(!file.remove("Dusk"));
            assert!(!file.remove("harbor"), "names match exactly");
            assert_eq!(names(&file), ["Harbor", "Dawn"]);
        }

        #[test]
        fn a_file_that_is_not_a_list_is_unreadable() {
            let dir = tempdir().unwrap();
            let path = dir.path().join("palettes.json");
            for text in ["{ not json", "{\"name\": \"Harbor\"}", "null"] {
                fs::write(&path, text).unwrap();
                assert_eq!(
                    PaletteFile::load(&path),
                    Err(ConfigError::UnreadablePalettes),
                    "{text}"
                );
            }
        }

        #[test]
        fn a_file_that_cannot_be_opened_is_unreadable() {
            let dir = tempdir().unwrap();
            assert_eq!(
                PaletteFile::load(dir.path()),
                Err(ConfigError::UnreadablePalettes)
            );
        }

        #[test]
        fn reports_a_file_it_cannot_save() {
            let dir = tempdir().unwrap();
            let file = PaletteFile {
                path: dir.path().join("missing_dir").join("palettes.json"),
                palettes: Vec::new(),
                skipped: Vec::new(),
            };
            assert_eq!(file.save(), Err(ConfigError::UnableToSavePalettes));
        }
    }

    mod name {
        use super::*;

        #[test]
        fn is_given_back_trimmed() {
            assert_eq!(palette_name("  Harbor \t", []), Ok("Harbor".to_string()));
        }

        #[test]
        fn must_not_be_blank() {
            assert_eq!(palette_name("", []), Err(PaletteNameError::Empty));
            assert_eq!(palette_name(" \t ", []), Err(PaletteNameError::Empty));
        }

        #[test]
        fn must_not_match_another_name_ignoring_case() {
            let taken = ["Navy & Champagne", "Nord"];
            assert_eq!(
                palette_name(" nord ", taken),
                Err(PaletteNameError::Taken("Nord".to_string()))
            );
            assert_eq!(
                palette_name("NAVY & CHAMPAGNE", taken),
                Err(PaletteNameError::Taken("Navy & Champagne".to_string()))
            );
        }

        #[test]
        fn folds_case_beyond_ascii() {
            assert_eq!(
                palette_name("ÉTÉ", ["été"]),
                Err(PaletteNameError::Taken("été".to_string()))
            );
        }

        #[test]
        fn may_resemble_another_name_without_matching_it() {
            assert_eq!(palette_name("Nord 2", ["Nord"]), Ok("Nord 2".to_string()));
        }

        #[test]
        fn errors_say_what_is_wrong() {
            assert_eq!(
                PaletteNameError::Empty.to_string(),
                "A palette needs a name"
            );
            assert_eq!(
                PaletteNameError::Taken("Nord".to_string()).to_string(),
                "There is already a palette called 'Nord'"
            );
        }
    }
}
