use eframe::*;
use egui::*;
use rfd::*;

use crate::character::{self, Character};
use crate::imaginators::{self, ImaginatorCrystal};
use crate::statictoys::{self, Expansion, Item};
use crate::vehicle::{self, Vehicle};
use crate::trap::{self, Trap};

use crate::skyfigures::{self, IntoEnumIterator};
use crate::skyhats::Hat;
use crate::skyutils::*;
use crate::skyvariants::Variant;

enum Optional {
    Character(character::Character),
    Vehicle(vehicle::Vehicle),
    Item(statictoys::Item),
    Expansion(statictoys::Expansion),
    Trap(trap::Trap),
    ImaginatorCrystal(imaginators::ImaginatorCrystal),
    Unknown(SkylanderBase),
    None
}

fn execute_generic<T: Default>(s: &mut Optional, f: &dyn Fn(&mut dyn Skylander) -> T) -> T {
    match s {
        Optional::Character(x) => f(x),
        Optional::Vehicle(x) => f(x),
        Optional::Trap(x) => f(x),
        Optional::ImaginatorCrystal(x) => f(x),
        Optional::Expansion(x) => f(x),
        Optional::Item(x) => f(x),
        Optional::Unknown(x) => f(x),
        Optional::None => T::default()
    }
}

pub struct SkyApp {
    toy: Optional,
    curr_file: Option<String>,
    top_query: String,
    inner_query: String,
    byte_start_idx: usize,
    bytes_val: usize,
    bytes_size: u8
}

impl SkyApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self { toy: Optional::None,
            curr_file: None,
            top_query: "".to_string(),
            inner_query: "".to_string(),
            byte_start_idx: 0,
            bytes_val: 0,
            bytes_size: 1
        }
    }

    fn button_per_it<T>(&mut self, ui: &mut Ui) where T: IntoEnumIterator + std::fmt::Display + Into<u16> + Copy {
        ui.text_edit_singleline(&mut self.top_query);
        ScrollArea::vertical().show(ui, |ui| {
            for i in T::iter() {
                let s = i.to_string();
                if s.to_lowercase().contains(&self.top_query.to_lowercase()) {
                    let _ = ui.menu_button(s, |ui| {
                        ui.text_edit_singleline(&mut self.inner_query);
                        ScrollArea::vertical().show(ui, |ui| {
                            for v in Variant::iter() {      // Update so variants correspond to figure
                                let si = v.to_string();
                                if si.to_lowercase().contains(&self.inner_query.to_lowercase())
                                        && ui.button(si).clicked() {
                                    self.toy = match Toy::try_from(i.into()).unwrap() {
                                        Toy::Character(x) => Optional::Character(Character::new(Toy::Character(x), v, None)),
                                        Toy::Vehicle(x) => Optional::Vehicle(Vehicle::new(Toy::Vehicle(x), v, None)),
                                        Toy::Item(x) => Optional::Item(Item::new(Toy::Item(x), v, None)),
                                        Toy::Expansion(x) => Optional::Expansion(Expansion::new(Toy::Expansion(x), v, None)),
                                        Toy::Trap(x) => Optional::Trap(Trap::new(Toy::Trap(x), v, None)),
                                        Toy::ImaginatorCrystal(x) => Optional::ImaginatorCrystal(ImaginatorCrystal::new(Toy::ImaginatorCrystal(x), v, None)),
                                        Toy::Unknown(x) => Optional::Unknown(SkylanderBase::new(Toy::Unknown(x), v, None))
                                    };
                                    self.inner_query = "".to_string();
                                    self.top_query = "".to_string();
                                    ui.close_menu();
                                }
                            }
                        });
                    });
                }
            }
        });
    }
    
    fn create_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Character", |ui| {
            self.button_per_it::<skyfigures::Character>(ui);
        });
        ui.menu_button("Item", |ui| {
            self.button_per_it::<skyfigures::Item>(ui);
        });
        ui.menu_button("Trap", |ui| {
            self.button_per_it::<skyfigures::Trap>(ui);
        });
        ui.menu_button("Vehicle", |ui| {
            self.button_per_it::<skyfigures::Vehicle>(ui);
        });
        ui.menu_button("Imaginator Crystal", |ui| {
            self.button_per_it::<skyfigures::ImaginatorCrystal>(ui);
        });
        ui.menu_button("Expansion", |ui| {
            self.button_per_it::<skyfigures::Expansion>(ui);
        });
    }

    fn upgrades_checkboxes(c: &mut character::Character, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Select upgrades: ");
            let mut arr = u8_bitmap_to_bool_arr(c.get_upgrades());
            for i in (0..8).rev() {
                ui.checkbox(&mut arr[i], "");
            }
            c.set_upgrades(bool_arr_to_u8_bitmap(&arr));
        });
    }

    fn upgrade_path_opts(c: &mut character::Character, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Choose upgrade path: ");
            let mut path = c.get_upgrade_path();
            ui.selectable_value(&mut path, UpgradePath::None, "None");
            ui.selectable_value(&mut path, UpgradePath::Top, "Top");
            ui.selectable_value(&mut path, UpgradePath::Bottom, "Bottom");
            c.set_upgrade_path(path);
        });
    }

    fn slider<T, S>(s: &mut S, ui: &mut Ui, getter: &dyn Fn(&S) -> T, setter: &dyn Fn(&mut S, T),
            min_val: T, max_val: T, label: &str) where T: eframe::emath::Numeric {
        ui.horizontal(|ui| {
            ui.label(label);
            let orig_value = getter(s);
            let mut val = orig_value;
            let slider = Slider::new(&mut val, min_val..=max_val).integer();
            ui.add(slider);
            if orig_value != val {
                setter(s, val);
            }
        });
    }

    fn hat_dropdown(c: &mut character::Character, ui: &mut Ui) {
        let mut hat = match c.get_hat() {
            Ok(h) => h,
            _ => Hat::None
        };
        ComboBox::new("hat dropdown", "Select a hat")
            .selected_text(hat.to_string())
            .show_ui(ui, |ui| {
                for h in Hat::iter() {
                    ui.selectable_value(&mut hat, h, h.to_string());
                }
            });
        c.set_hat(hat);
    }

    fn performance_upg_dropdown(v: &mut vehicle::Vehicle, ui: &mut Ui) {
        use vehicle::PerformanceUpgrade::*;
        let orig_upg = match v.get_performance_upgrade() {
            Ok(h) => h,
            _ => First
        };
        let mut upg = orig_upg;
        ComboBox::new("perf. upgrade dropdown", "Select a performance upgrade")
            .selected_text(upg.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut upg, First, First.to_string());
                ui.selectable_value(&mut upg, Second, Second.to_string());
                ui.selectable_value(&mut upg, Third, Third.to_string());
                ui.selectable_value(&mut upg, Fourth, Fourth.to_string());
            });
        if orig_upg != upg {
            v.set_performance_upgrade(upg);
        }
    }

    fn set_bytes_widget(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ComboBox::new("bytes size", "")
                .selected_text(
                    match self.bytes_size {
                        1 => "u8",
                        2 => "u16",
                        4 => "u32",
                        8 => "u64",
                        _ => "unexpected value"
                    }
                ).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.bytes_size, 1, "u8");
                    ui.selectable_value(&mut self.bytes_size, 2, "u16");
                    ui.selectable_value(&mut self.bytes_size, 4, "u32");
                    ui.selectable_value(&mut self.bytes_size, 8, "u64");
                });
            
            ui.label("Enter a start byte [0, 1024): ");
            ui.add(egui::DragValue::new(&mut self.byte_start_idx));
            ui.label("Enter a little-endian value: ");
            ui.add(egui::DragValue::new(&mut self.bytes_val));

            if ui.button("Write").clicked() {
                let bytes = match self.bytes_size {
                    1 => vec![(self.bytes_val as u8)],
                    2 => (self.bytes_val as u16).to_le_bytes().to_vec(),
                    4 => (self.bytes_val as u32).to_le_bytes().to_vec(),
                    8 => (self.bytes_val as u64).to_le_bytes().to_vec(),
                    _ => Vec::new()
                };

                execute_generic(&mut self.toy, &|x| { x.set_bytes(self.byte_start_idx, &bytes); });
            }
        });
    }

}

fn from_base(s: SkylanderBase) -> Optional {
    match s.get_figure() {
        Toy::Character(_) => Optional::Character(character::Character::from(s)),
        Toy::Vehicle(_) => Optional::Vehicle(vehicle::Vehicle::from(s)),
        Toy::Trap(_) => Optional::Trap(trap::Trap::from(s)),
        Toy::Item(_) => Optional::Item(statictoys::Item::from(s)),
        Toy::Expansion(_) => Optional::Expansion(statictoys::Expansion::from(s)),
        Toy::ImaginatorCrystal(_) => Optional::ImaginatorCrystal(imaginators::ImaginatorCrystal::from(s)),
        Toy::Unknown(_) => Optional::Unknown(s)
    }
}

fn to_base(s: Optional) -> Option<SkylanderBase> {
    match s {
        Optional::Character(x) => Some(x.into()),
        Optional::Vehicle(x) => Some(x.into()),
        Optional::Trap(x) => Some(x.into()),
        Optional::Expansion(x) => Some(x.into()),
        Optional::Item(x) => Some(x.into()),
        Optional::ImaginatorCrystal(x) => Some(x.into()),
        Optional::Unknown(x) => Some(x),
        Optional::None => None
    }
}

impl App for SkyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("Options").show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.horizontal_wrapped(|ui| {
                ui.menu_button("Create new", |ui| {self.create_menu(ui)});
                
                if ui.button("Open File").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        let selected_file = path.display().to_string();
                        if let Ok(s) = SkylanderBase::from_filename(&selected_file) {
                            self.toy = from_base(s);
                            self.curr_file = Some(selected_file);
                        }
                    }
                }
                
                if self.curr_file.is_some() {
                    if ui.button("Save").clicked() {
                        execute_generic(&mut self.toy, &|x| {
                            let _ = x.save_to_filename(&self.curr_file.as_ref().unwrap());
                        });
                    }
                }

                if ui.button("Save As File").clicked() {
                    match &self.toy {
                        Optional::None => (),
                        _ => {
                            if let Some(path) = FileDialog::new().save_file() {
                                let selected_file = path.display().to_string();
                                execute_generic(&mut self.toy, &|x| {
                                    let _ = x.save_to_filepath(&path);
                                });
                                self.curr_file = Some(selected_file);
                            }
                        }
                    }
                }

                if ui.button("Scan NFC").clicked() {
                    if let Ok(s) = SkylanderBase::from_nfc() {
                        self.toy = from_base(s);
                    }
                }
    
                if ui.button("Save to NFC").clicked() {
                    match &self.toy {
                        Optional::None => (),
                        _ => {
                            if let Some(path) = FileDialog::new().save_file() {
                                let selected_file = path.display().to_string();
                                execute_generic(&mut self.toy, &|x| {
                                    let _ = x.save_to_nfc();
                                });
                                self.curr_file = Some(selected_file);
                            }
                        }
                    }
                }

            });
        });

        SidePanel::left("Figure").resizable(true).show(ctx, |ui| {
            ui.label(format!("Current toy: {}, Variant: {}",
                {
                    execute_generic(&mut self.toy, &|x| { x.get_figure().to_string() })
                },
                {
                    execute_generic(&mut self.toy, &|x| { x.get_variant().to_string() })
                }
            ));

            if ui.button("Clear toy").clicked() {
                self.toy = Optional::None;
                self.curr_file = None;
            }
        });
        CentralPanel::default().show(ctx, |ui| {
            // Type-dependent behavior
            match &mut self.toy {
                Optional::Character(c) => {
                    SkyApp::upgrades_checkboxes(c, ui);
                    SkyApp::upgrade_path_opts(c, ui);
                    { // Wowpow checkbox
                        let mut wowpow = c.get_wowpow();
                        ui.checkbox(&mut wowpow, "Set wowpow");
                        c.set_wowpow(wowpow);
                    }
                    SkyApp::slider(c, ui, &Character::get_xp, &Character::set_xp, 0, 197500u32, "Set xp: ");
                    SkyApp::slider(c, ui, &Character::get_level, &Character::set_level, 1, 20, "Set level: ");
                    SkyApp::slider(c, ui, &Character::get_gold, &Character::set_gold, 0, 0xFFFFu16, "Set gold: ");
                    SkyApp::hat_dropdown(c, ui);
                },
                Optional::Vehicle(v) => {
                    SkyApp::slider(v, ui, &vehicle::Vehicle::get_gears, &vehicle::Vehicle::set_gears, 0u16, 33000, "Set gears: ");
                    SkyApp::performance_upg_dropdown(v, ui);
                },
                _ => ()
            };

            // General Behavior
            match &mut self.toy {
                Optional::None => (),
                _ => {
                    self.set_bytes_widget(ui);

                    if ui.button("Reset figure").clicked() {
                        execute_generic(&mut self.toy, &|x| { x.clear(); });
                    }
                }
            }
        });
    }
}

fn u8_bitmap_to_bool_arr(bitmap: u8) -> [bool; 8] {
    let mut arr = [false; 8];
    for i in 0..8 {
        arr[i] = (bitmap >> 8 - i - 1 & 1) == 1;
    }

    arr
}

fn bool_arr_to_u8_bitmap(arr: &[bool; 8]) -> u8 {
    let mut bitmap = 0u8;
    for i in 0..8 {
        bitmap <<= 1;
        bitmap |= arr[i] as u8;
    }

    bitmap
}

#[test]
fn test_bool_arr_to_u8_bitmap() {
    let arr1 = [false; 8];
    let map1 = bool_arr_to_u8_bitmap(&arr1);
    assert_eq!(map1, 0);
    
    let arr2 = [true; 8];
    let map2 = bool_arr_to_u8_bitmap(&arr2);
    assert_eq!(map2, 0xFFu8);
}