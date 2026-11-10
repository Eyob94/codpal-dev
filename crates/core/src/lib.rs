//! All business logic lives here. Pure functions only (no IO, no async), so the
//! exact same code runs in the server and, through `crates/wasm`, in the browser.

use api_types::{Badge, CreateItem, FieldError, Item, ItemView};

pub const MAX_TITLE_LEN: usize = 120;

/// Validate and normalise a create-item intent.
pub fn validate_create_item(input: &CreateItem) -> Result<CreateItem, Vec<FieldError>> {
    let title = input.title.trim();
    let mut errors = Vec::new();
    if title.is_empty() {
        errors.push(FieldError {
            field: "title".into(),
            message: "Title is required".into(),
        });
    } else if title.chars().count() > MAX_TITLE_LEN {
        errors.push(FieldError {
            field: "title".into(),
            message: format!("Title must be at most {MAX_TITLE_LEN} characters"),
        });
    }
    if errors.is_empty() {
        Ok(CreateItem {
            title: title.to_owned(),
        })
    } else {
        Err(errors)
    }
}

/// Turn a domain item into exactly what the UI should render.
pub fn item_view(item: &Item) -> ItemView {
    ItemView {
        id: item.id.to_string(),
        label: item.title.clone(),
        badge: if item.done { Badge::Done } else { Badge::Open },
        can_complete: !item.done,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn rejects_blank_title() {
        let r = validate_create_item(&CreateItem {
            title: "   ".into(),
        });
        assert_eq!(r.unwrap_err()[0].field, "title");
    }

    #[test]
    fn trims_title() {
        let r = validate_create_item(&CreateItem {
            title: "  hi  ".into(),
        })
        .unwrap();
        assert_eq!(r.title, "hi");
    }

    #[test]
    fn done_items_cannot_be_completed() {
        let item = Item {
            id: Uuid::nil(),
            title: "x".into(),
            done: true,
            created_at: Utc::now(),
        };
        assert!(!item_view(&item).can_complete);
    }
}
