mod imp;

use convert_case::{Case, Casing};
use exif::In;
use gtk4::{glib, prelude::TreeViewExt, ListStore};

use crate::image::Image;

glib::wrapper! {
pub struct InfoView(ObjectSubclass<imp::InfoViewImp>)
    @extends gtk4::Widget, gtk4::TreeView, gtk4::Scrollable;
}

#[derive(Debug)]
#[repr(u32)]
pub enum Columns {
    Key = 0,
    Value,
}

impl Columns {
    fn store() -> ListStore {
        let col_types: [glib::Type; 2] = [glib::Type::STRING, glib::Type::STRING];
        ListStore::new(&col_types)
    }
}

impl InfoView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for InfoView {
    fn default() -> Self {
        Self::new()
    }
}

fn insert(store: &ListStore, key: &str, value: &str) {
    store.insert_with_values(
        None,
        &[(Columns::Key as u32, &key), (Columns::Value as u32, &value)],
    );
}

impl InfoView {
    pub fn update(&self, image: &Image) {
        let store = Columns::store();

        match &image.pixbuf {
            Some(pixbuf) => {
                insert(&store, "width", &format!("{} px", pixbuf.width()));
                insert(&store, "height", &format!("{} px", pixbuf.height()));
                insert(
                    &store,
                    "alpha channel",
                    if pixbuf.has_alpha() { "yes" } else { "no" },
                );
            }
            None => insert(&store, "", "no image"),
        }

        match &image.exif {
            Some(exif) => {
                for f in exif.fields() {
                    if f.ifd_num == In::PRIMARY {
                        let key = f.tag.to_string();
                        let key = key.from_case(Case::Pascal).to_case(Case::Lower);
                        insert(&store, &key, &f.display_value().with_unit(exif).to_string())
                    }
                }
            }
            None => {
                // println!("No exif data");
            }
        }
        self.set_model(Some(&store));
    }
}
