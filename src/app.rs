use eframe::*;
use egui::*;
use rfd::*;

use crate::skyhats::Hat;
use crate::skyutils::*;
use crate::skyfigures::*;
use crate::skyvariants::Variant;
// use crate::skyvariants::*;

pub struct SkyApp {
    toy: Option<Skylander>,
    curr_file: Option<String>,
    top_query: String,
    inner_query: String,
}

impl SkyApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self { toy: None, curr_file: None, top_query: "".to_string(), inner_query: "".to_string() }
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
                                    self.toy = Some(Skylander::new(i, v, None));
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
            self.button_per_it::<Character>(ui);
        });
        ui.menu_button("Item", |ui| {
            self.button_per_it::<Item>(ui);
        });
        ui.menu_button("Trap", |ui| {
            self.button_per_it::<Trap>(ui);
        });
        ui.menu_button("Vehicle", |ui| {
            self.button_per_it::<Vehicle>(ui);
        });
        ui.menu_button("Imaginator Crystal", |ui| {
            self.button_per_it::<ImaginatorCrystal>(ui);
        });
        ui.menu_button("Expansion", |ui| {
            self.button_per_it::<Expansion>(ui);
        });
    }

    fn upgrades_checkboxes(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Select upgrades: ");
            let mut arr = u8_bitmap_to_bool_arr(self.toy.as_ref().unwrap().get_upgrades());
            for i in 0..8 {
                ui.checkbox(&mut arr[i], "");
            }
            self.toy.as_mut().unwrap().set_upgrades(bool_arr_to_u8_bitmap(&arr));
        });
    }

    fn upgrade_path_opts(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Choose upgrade path: ");
            let mut path = self.toy.as_ref().unwrap().get_upgrade_path();
            ui.selectable_value(&mut path, UpgradePath::None, "None");
            ui.selectable_value(&mut path, UpgradePath::Top, "Top");
            ui.selectable_value(&mut path, UpgradePath::Bottom, "Bottom");
            self.toy.as_mut().unwrap().set_upgrade_path(path);
        });
    }

    fn slider<T>(&mut self, ui: &mut Ui, getter: &dyn Fn(&Skylander) -> T, setter: &dyn Fn(&mut Skylander, T),
            min_val: T, max_val: T, label: &str) where T: eframe::emath::Numeric {
        ui.horizontal(|ui| {
            ui.label(label);
            let mut val = getter(self.toy.as_ref().unwrap());
            let slider = Slider::new(&mut val, min_val..=max_val).integer();
            ui.add(slider);
            setter(self.toy.as_mut().unwrap(), val);
        });
    }

    fn hat_dropdown(&mut self, ui: &mut Ui) {
        let mut hat = match self.toy.as_ref().unwrap().get_hat() {
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
        self.toy.as_mut().unwrap().set_hat(hat);
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
                        self.toy = match Skylander::from_filename(&selected_file) {
                            Ok(s) => Some(s),
                            _ => None
                        };
                        self.curr_file = Some(selected_file);
                    }
                }
                
                if self.curr_file.is_some() {
                    if ui.button("Save").clicked() {
                        let _ = self.toy.as_ref().unwrap().save_to_filename(&self.curr_file.as_ref().unwrap());
                    }
                }

                if ui.button("Save As File").clicked() {
                    if self.toy.is_some() {
                        if let Some(path) = FileDialog::new().save_file() {
                            let selected_file = path.display().to_string();
                            let _ = self.toy.as_ref().unwrap().save_to_filename(&selected_file);
                            self.curr_file = Some(selected_file);
                        }
                    }
                }

                if ui.button("Scan NFC").clicked() {
                    self.toy = match Skylander::from_nfc() {
                        Ok(s) => Some(s),
                        _ => { println!("failed to read"); None }
                    };
                }
    
                if ui.button("Save to NFC").clicked() {
                    if self.toy.is_some() {
                        let _ = self.toy.as_ref().unwrap().save_to_nfc();
                    }
                }

            });
        });

        SidePanel::left("Figure").resizable(true).show(ctx, |ui| {
            ui.label(format!("Current toy: {}, Variant: {}", match self.toy.as_ref() {
                Some(t) => t.get_figure().to_string(),
                None => "None".to_string()
            },
                match self.toy.as_ref() {
                    Some(t) => match t.get_variant() {
                        Ok(v) => v.to_string(),
                        _ => "None".to_string()
                    },
                    None => "None".to_string()
                }
            ));

            if ui.button("Clear toy").clicked() {
                self.toy = None;
                self.curr_file = None;
            }
        });
        CentralPanel::default().show(ctx, |ui| {
            if self.toy.is_some() {
                self.upgrades_checkboxes(ui);
                self.upgrade_path_opts(ui);
                { // Wowpow checkbox
                    let mut wowpow = self.toy.as_ref().unwrap().get_wowpow();
                    ui.checkbox(&mut wowpow, "Set wowpow");
                    self.toy.as_mut().unwrap().set_wowpow(wowpow);
                }

                self.slider(ui, &Skylander::get_xp, &Skylander::set_xp, 0, 197500u32, "Set xp: ");
                self.slider(ui, &Skylander::get_level, &Skylander::set_level, 1, 20, "Set level: ");
                self.slider(ui, &Skylander::get_gold, &Skylander::set_gold, 0, 0xFFFFu16, "Set gold: ");
                self.hat_dropdown(ui);                
                
                if ui.button("Reset figure").clicked() {
                    self.toy.as_mut().unwrap().clear();
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