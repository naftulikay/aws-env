#[cfg(test)]
mod tests;

use indexmap::IndexMap;
use ini_core::{Item as IniEntity, Parser as IniParser};

/// A zero-copy, parsed INI file represented as a map of section names to maps of key/value properties.
///
/// Internally, it is an `IndexMap`, so it will respect section order.
pub type Ini<'a> = IndexMap<&'a str, IniSection<'a>>;
/// A zero-copy INI file section represented as a map of keys to optional values.
///
/// If a key exists in this map but its value is `None`, this means that either the value was empty or the key had no
/// equals-sign associated with it, in `ini_core` parlance, this is an `ini_core::Item::Action`.
///
/// Internally, it is an `IndexMap`, so it will respect key order.
pub type IniSection<'a> = IndexMap<&'a str, Option<&'a str>>;

/// Parse a string reference into an `Ini` map of `IniSections`.
///
/// Generics make it possible to pass a `&String`, `&str`, or even a `&Cow<str>`.
pub fn parse<'a, S: AsRef<str> + ?Sized>(source: &'a S) -> Ini<'a> {
    // NOTE we assume that a given file won't have more than 16 sections, so overallocate early.
    let mut map = Ini::with_capacity(16);
    let parser = IniParser::new(source.as_ref()).auto_trim(true);

    let mut current_section = None;

    for (_line, entity) in parser.enumerate() {
        match entity {
            IniEntity::Comment(_) | IniEntity::Blank => (),
            IniEntity::Error(e) => {
                log::warn!("Error parsing INI file: {}", e);
            }
            IniEntity::Section(section) => {
                current_section = Some(section);
                // NOTE we assume that each section will generally have no more than four keys
                map.insert(section, IniSection::with_capacity(4));
            }
            IniEntity::Action(key) => {
                if let Some(section) = current_section {
                    map.get_mut(section).unwrap().insert(key, None);
                }
            }
            IniEntity::Property(key, value) => {
                if let Some(section) = current_section {
                    map.get_mut(section).unwrap().insert(key, Some(value));
                }
            }
        }
    }

    map
}
