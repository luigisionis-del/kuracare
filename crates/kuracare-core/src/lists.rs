//! User-editable presets and labels. Values come from Luigi, never invented.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::kinds::Route;
use crate::store::Store;
use crate::vault::Vault;

pub const LISTS_ID: &str = "cfg:lists";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StrainGroup {
    Thc,
    Cbd,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrainPreset {
    pub id: String,
    pub name: String,
    pub group: StrainGroup,
    pub thc_pct: Option<f64>,
    pub cbd_pct: Option<f64>,
    pub amount: Option<f64>,
    pub unit: String,
    pub route: Option<Route>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplementPreset {
    pub id: String,
    pub name: String,
    pub amount: Option<f64>,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lists {
    pub version: u32,
    pub strains: Vec<StrainPreset>,
    pub supplements: Vec<SupplementPreset>,
    pub care_labels: Vec<String>,
    pub thought_labels: Vec<String>,
    #[serde(default = "default_rooms")]
    pub rooms: Vec<String>,
    #[serde(default = "default_home_activities")]
    pub home_activities: Vec<String>,
}

fn strings(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| (*s).to_owned()).collect()
}

fn default_rooms() -> Vec<String> {
    strings(&["Bagno", "Cucina", "Camera da letto"])
}

fn default_home_activities() -> Vec<String> {
    strings(&["Riordino", "Pulizia", "Bucato", "Cambio lenzuola", "Piatti"])
}

pub fn new_id(prefix: &str) -> String {
    format!(
        "{prefix}-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    )
}

impl Default for Lists {
    fn default() -> Self {
        let strain = |id: &str, name: &str, group| StrainPreset {
            id: id.into(),
            name: name.into(),
            group,
            thc_pct: None,
            cbd_pct: None,
            amount: None,
            unit: "g".into(),
            route: None,
        };
        let supp = |id: &str, name: &str| SupplementPreset {
            id: id.into(),
            name: name.into(),
            amount: None,
            unit: "mg".into(),
        };
        Self {
            version: 1,
            strains: vec![
                strain(
                    "s-sherbinskis",
                    "Sherbinskis × Silent Seeds",
                    StrainGroup::Thc,
                ),
                strain("s-mandarine", "Mandarine Flokies", StrainGroup::Cbd),
                strain("s-lemon", "Lemon Cockies", StrainGroup::Cbd),
            ],
            supplements: vec![
                supp("i-cbd", "CBD"),
                supp("i-agpc", "Alfa-GPC"),
                supp("i-glicina", "Glicina"),
                supp("i-colina", "Colina"),
            ],
            care_labels: strings(&[
                "Passeggiata",
                "Movimento",
                "Pratica corporea",
                "Natura",
                "Riposo",
                "Sonno buono",
                "Un buon pasto",
                "Contatto con le persone",
            ]),
            thought_labels: strings(&[
                "Rimuginio",
                "Preoccupazione",
                "Autocritica",
                "Un ricordo che torna",
                "Altro",
            ]),
            rooms: default_rooms(),
            home_activities: default_home_activities(),
        }
    }
}

pub fn load<S: Store>(v: &Vault<S>) -> Result<Lists> {
    Ok(v.get(LISTS_ID)?.unwrap_or_default())
}

pub fn save<S: Store>(v: &Vault<S>, lists: &Lists) -> Result<()> {
    v.put(LISTS_ID, lists)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_carry_no_invented_values() {
        let l = Lists::default();
        assert!(l.strains.iter().all(|s| s.thc_pct.is_none()
            && s.cbd_pct.is_none()
            && s.amount.is_none()
            && s.route.is_none()));
        assert!(l.supplements.iter().all(|s| s.amount.is_none()));
        assert_eq!(l.rooms, vec!["Bagno", "Cucina", "Camera da letto"]);
    }

    #[test]
    fn older_lists_without_rooms_get_the_defaults() {
        let mut v = serde_json::to_value(Lists::default()).unwrap();
        v.as_object_mut().unwrap().remove("rooms");
        let l: Lists = serde_json::from_value(v).unwrap();
        assert_eq!(l.rooms.len(), 3);
    }
}
