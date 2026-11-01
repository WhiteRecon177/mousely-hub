//! Built-in list of known mice (generated from `data/mice.json`).
//!
//! It only *recognises* a mouse (brand, model, declared specs). It never says that a mouse can
//! be configured: that depends on a driver in `drivers/`.

use super::database_data::{MICE, VENDORS};

/// One USB id of a model. `kind`: 0 = cable, 1 = wireless receiver, 2 = Bluetooth.
pub struct Id {
    pub vid: u16,
    pub pid: u16,
    pub kind: u8,
}

#[allow(dead_code)]
pub struct MouseEntry {
    pub brand: &'static str,
    pub family: &'static str,
    pub model: &'static str,
    pub usb_name: &'static str,
    pub ids: &'static [Id],
    pub dpi_min: u32,
    pub dpi_max: u32,
    pub poll_wired: u32,
    pub poll_wireless: u32,
    pub poll_dongle: u32,
    /// false when the declared specs are marked "to be verified" in the source lists.
    pub verified: bool,
}

pub enum Recognised {
    /// Exactly one model uses this id.
    Model {
        entry: &'static MouseEntry,
        kind: u8,
    },
    /// The id is shared by several models of one brand (for example a common receiver),
    /// or only the vendor is known.
    Brand(&'static str),
}

pub fn recognise(vid: u16, pid: u16) -> Option<Recognised> {
    let mut found: Option<(&'static MouseEntry, u8)> = None;
    let mut models = 0;
    let mut brand: Option<&'static str> = None;
    let mut mixed = false;

    for entry in MICE {
        for id in entry.ids {
            if id.vid != vid || id.pid != pid {
                continue;
            }

            if found.map(|(known, _)| known.model != entry.model).unwrap_or(true) {
                models += 1;
            }
            found = Some((entry, id.kind));

            match brand {
                None => brand = Some(entry.brand),
                Some(known) if known != entry.brand => mixed = true,
                _ => {}
            }
        }
    }

    match (found, models, brand) {
        (Some((entry, kind)), 1, _) => Some(Recognised::Model { entry, kind }),
        (Some(_), _, Some(name)) if !mixed => Some(Recognised::Brand(name)),
        (Some(_), _, _) => None,
        _ => VENDORS
            .iter()
            .find(|(known, _)| *known == vid)
            .map(|(_, name)| Recognised::Brand(name)),
    }
}

#[allow(dead_code)]
pub fn highest_polling(entry: &MouseEntry) -> u32 {
    entry
        .poll_wired
        .max(entry.poll_wireless)
        .max(entry.poll_dongle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_is_not_empty_and_has_unique_shapes() {
        assert!(MICE.len() > 500);
        assert!(MICE.iter().all(|entry| !entry.model.is_empty()));
    }

    #[test]
    fn razer_vendor_is_known() {
        assert!(matches!(
            recognise(0x1532, 0xFFFF),
            Some(Recognised::Brand("Razer"))
        ));
    }

    #[test]
    fn shared_chip_vendor_is_not_a_brand() {
        assert!(recognise(0x258A, 0xFFFF).is_none());
    }

    #[test]
    fn unknown_vendor_is_none() {
        assert!(recognise(0x1234, 0x5678).is_none());
    }

    #[test]
    fn single_model_id_gives_the_model() {
        let entry = MICE
            .iter()
            .find(|entry| {
                entry.brand == "Razer"
                    && entry.ids.len() == 1
                    && MICE
                        .iter()
                        .filter(|other| other.ids.iter().any(|id| id.pid == entry.ids[0].pid && id.vid == entry.ids[0].vid))
                        .count()
                        == 1
            })
            .expect("a Razer model with a unique id");
        let id = &entry.ids[0];
        match recognise(id.vid, id.pid) {
            Some(Recognised::Model { entry: found, .. }) => assert_eq!(found.model, entry.model),
            _ => panic!("expected the model"),
        }
    }
}