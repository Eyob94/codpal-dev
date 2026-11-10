//! All business logic lives here. Pure functions only (no IO, no async), so the
//! exact same code runs in the server and, through `crates/wasm`, in the browser.

use api_types::{
    Badge, CreateItem, CreateProject, FieldError, Item, ItemView, Project, ProjectPatch,
    ProjectView, UpdateProject,
};

pub const MAX_TITLE_LEN: usize = 120;
pub const MAX_PROJECT_NAME_LEN: usize = 100;
pub const MAX_PROJECT_DESCRIPTION_LEN: usize = 1000;

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

/// Validate and normalise a create-project intent. Returns all errors at once.
pub fn validate_create_project(_input: &CreateProject) -> Result<CreateProject, Vec<FieldError>> {
    todo!("Code issue #3")
}

/// Validate and normalise a partial update. All-`None` input is a valid no-op.
pub fn validate_update_project(_input: &UpdateProject) -> Result<ProjectPatch, Vec<FieldError>> {
    todo!("Code issue #3")
}

/// Map a domain project to the view model the API returns.
pub fn project_view(_project: &Project) -> ProjectView {
    todo!("Code issue #3")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
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

    // ---- helpers for project tests ------------------------------------------

    fn create(name: &str) -> CreateProject {
        CreateProject {
            name: name.into(),
            repo_url: None,
            description: None,
        }
    }

    fn update() -> UpdateProject {
        UpdateProject {
            name: None,
            repo_url: None,
            description: None,
        }
    }

    fn fields(errs: &[FieldError]) -> Vec<&str> {
        errs.iter().map(|e| e.field.as_str()).collect()
    }

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .unwrap()
            .with_timezone(&Utc)
    }

    // ---- A. create validation -------------------------------------------------

    #[test]
    fn create_accepts_name_only() {
        let out = validate_create_project(&create("Alpha")).unwrap();
        assert_eq!(out.name, "Alpha");
        assert_eq!(out.repo_url, None);
        assert_eq!(out.description, None);
    }

    #[test]
    fn create_accepts_all_fields() {
        let out = validate_create_project(&CreateProject {
            name: "Alpha".into(),
            repo_url: Some("https://github.com/a/b".into()),
            description: Some("A project".into()),
        })
        .unwrap();
        assert_eq!(out.repo_url.as_deref(), Some("https://github.com/a/b"));
        assert_eq!(out.description.as_deref(), Some("A project"));
    }

    #[test]
    fn create_trims_name() {
        let out = validate_create_project(&create("  foo  ")).unwrap();
        assert_eq!(out.name, "foo");
    }

    #[test]
    fn create_rejects_blank_name() {
        let errs = validate_create_project(&create("   ")).unwrap_err();
        assert_eq!(fields(&errs), vec!["name"]);
    }

    #[test]
    fn create_rejects_empty_name() {
        let errs = validate_create_project(&create("")).unwrap_err();
        assert_eq!(fields(&errs), vec!["name"]);
    }

    #[test]
    fn create_accepts_name_of_100_chars() {
        let out = validate_create_project(&create(&"a".repeat(100))).unwrap();
        assert_eq!(out.name, "a".repeat(100));
    }

    #[test]
    fn create_rejects_name_of_101_chars() {
        let errs = validate_create_project(&create(&"a".repeat(101))).unwrap_err();
        assert_eq!(fields(&errs), vec!["name"]);
    }

    #[test]
    fn create_counts_name_length_in_characters_not_bytes() {
        // 100 emoji = 400 bytes but 100 characters: must be accepted.
        let name = "🙂".repeat(100);
        let out = validate_create_project(&create(&name)).unwrap();
        assert_eq!(out.name, name);
    }

    #[test]
    fn create_rejects_101_emoji_in_name() {
        let errs = validate_create_project(&create(&"🙂".repeat(101))).unwrap_err();
        assert_eq!(fields(&errs), vec!["name"]);
    }

    #[test]
    fn create_accepts_repo_url_forms() {
        for url in [
            "https://github.com/a/b",
            "http://x.com/a.git",
            "ssh://git@x.com/a.git",
            "git@github.com:a/b.git",
        ] {
            let input = CreateProject {
                name: "p".into(),
                repo_url: Some(url.into()),
                description: None,
            };
            let out = validate_create_project(&input)
                .unwrap_or_else(|e| panic!("{url} should be accepted, got {e:?}"));
            assert_eq!(out.repo_url.as_deref(), Some(url));
        }
    }

    #[test]
    fn create_rejects_invalid_repo_url() {
        for url in [
            "ftp://x",
            "not a url",
            "github.com/a/b",
            "javascript:alert(1)",
        ] {
            let input = CreateProject {
                name: "p".into(),
                repo_url: Some(url.into()),
                description: None,
            };
            let errs =
                validate_create_project(&input).expect_err(&format!("{url} should be rejected"));
            assert_eq!(fields(&errs), vec!["repo_url"], "for {url}");
        }
    }

    #[test]
    fn create_trims_repo_url() {
        let input = CreateProject {
            name: "p".into(),
            repo_url: Some("  https://github.com/a/b  ".into()),
            description: None,
        };
        let out = validate_create_project(&input).unwrap();
        assert_eq!(out.repo_url.as_deref(), Some("https://github.com/a/b"));
    }

    #[test]
    fn create_turns_empty_repo_url_into_none() {
        let input = CreateProject {
            name: "p".into(),
            repo_url: Some("".into()),
            description: None,
        };
        let out = validate_create_project(&input).unwrap();
        assert_eq!(out.repo_url, None);
    }

    #[test]
    fn create_accepts_description_of_1000_chars() {
        let input = CreateProject {
            name: "p".into(),
            repo_url: None,
            description: Some("d".repeat(1000)),
        };
        let out = validate_create_project(&input).unwrap();
        assert_eq!(out.description, Some("d".repeat(1000)));
    }

    #[test]
    fn create_rejects_description_of_1001_chars() {
        let input = CreateProject {
            name: "p".into(),
            repo_url: None,
            description: Some("d".repeat(1001)),
        };
        let errs = validate_create_project(&input).unwrap_err();
        assert_eq!(fields(&errs), vec!["description"]);
    }

    #[test]
    fn create_trims_description() {
        let input = CreateProject {
            name: "p".into(),
            repo_url: None,
            description: Some("  hi  ".into()),
        };
        let out = validate_create_project(&input).unwrap();
        assert_eq!(out.description.as_deref(), Some("hi"));
    }

    #[test]
    fn create_turns_blank_description_into_none() {
        let input = CreateProject {
            name: "p".into(),
            repo_url: None,
            description: Some("   ".into()),
        };
        let out = validate_create_project(&input).unwrap();
        assert_eq!(out.description, None);
    }

    #[test]
    fn create_reports_all_bad_fields_at_once() {
        let input = CreateProject {
            name: "".into(),
            repo_url: Some("bad".into()),
            description: Some("d".repeat(1001)),
        };
        let errs = validate_create_project(&input).unwrap_err();
        assert_eq!(fields(&errs), vec!["name", "repo_url", "description"]);
    }

    #[test]
    fn create_rejects_nul_byte_in_any_text_field() {
        // Postgres cannot store NUL in text, so this must be a validation error, not a 500.
        let input = CreateProject {
            name: "a\0b".into(),
            repo_url: Some("https://x.com/\0".into()),
            description: Some("x\0y".into()),
        };
        let errs = validate_create_project(&input).unwrap_err();
        assert_eq!(fields(&errs), vec!["name", "repo_url", "description"]);
    }

    // ---- A. update validation -------------------------------------------------

    #[test]
    fn update_with_no_fields_is_a_noop() {
        let patch = validate_update_project(&update()).unwrap();
        assert_eq!(patch, ProjectPatch::default());
    }

    #[test]
    fn update_leaves_unprovided_fields_alone() {
        let input = UpdateProject {
            description: Some("ok".into()),
            ..update()
        };
        let patch = validate_update_project(&input).unwrap();
        assert_eq!(patch.name, None);
        assert_eq!(patch.repo_url, None);
        assert_eq!(patch.description, Some(Some("ok".into())));
    }

    #[test]
    fn update_rejects_blank_name() {
        let input = UpdateProject {
            name: Some("   ".into()),
            ..update()
        };
        let errs = validate_update_project(&input).unwrap_err();
        assert_eq!(fields(&errs), vec!["name"]);
    }

    #[test]
    fn update_rejects_name_of_101_chars() {
        let input = UpdateProject {
            name: Some("a".repeat(101)),
            ..update()
        };
        let errs = validate_update_project(&input).unwrap_err();
        assert_eq!(fields(&errs), vec!["name"]);
    }

    #[test]
    fn update_trims_name() {
        let input = UpdateProject {
            name: Some("  foo  ".into()),
            ..update()
        };
        let patch = validate_update_project(&input).unwrap();
        assert_eq!(patch.name, Some("foo".into()));
    }

    #[test]
    fn update_rejects_invalid_repo_url() {
        let input = UpdateProject {
            repo_url: Some("github.com/a/b".into()),
            ..update()
        };
        let errs = validate_update_project(&input).unwrap_err();
        assert_eq!(fields(&errs), vec!["repo_url"]);
    }

    #[test]
    fn update_empty_repo_url_clears_it() {
        let input = UpdateProject {
            repo_url: Some("".into()),
            ..update()
        };
        let patch = validate_update_project(&input).unwrap();
        assert_eq!(patch.repo_url, Some(None));
    }

    #[test]
    fn update_trims_repo_url() {
        let input = UpdateProject {
            repo_url: Some("  https://github.com/a/b  ".into()),
            ..update()
        };
        let patch = validate_update_project(&input).unwrap();
        assert_eq!(patch.repo_url, Some(Some("https://github.com/a/b".into())));
    }

    #[test]
    fn update_empty_description_clears_it() {
        let input = UpdateProject {
            description: Some("".into()),
            ..update()
        };
        let patch = validate_update_project(&input).unwrap();
        assert_eq!(patch.description, Some(None));
    }

    #[test]
    fn update_rejects_description_of_1001_chars() {
        let input = UpdateProject {
            description: Some("d".repeat(1001)),
            ..update()
        };
        let errs = validate_update_project(&input).unwrap_err();
        assert_eq!(fields(&errs), vec!["description"]);
    }

    // ---- A. view --------------------------------------------------------------

    fn sample_project(repo_url: Option<&str>, description: Option<&str>) -> Project {
        Project {
            id: Uuid::parse_str("0190f2a4-7c3b-7d4e-8f5a-1b2c3d4e5f60").unwrap(),
            name: "Alpha".into(),
            repo_url: repo_url.map(Into::into),
            description: description.map(Into::into),
            created_at: at("2026-01-02T03:04:05Z"),
            updated_at: at("2026-01-02T04:05:06.123456Z"),
        }
    }

    #[test]
    fn project_view_maps_every_field() {
        let p = sample_project(Some("https://github.com/a/b"), Some("desc"));
        let view = project_view(&p);
        assert_eq!(view.id, "0190f2a4-7c3b-7d4e-8f5a-1b2c3d4e5f60");
        assert_eq!(view.name, "Alpha");
        assert_eq!(view.repo_url.as_deref(), Some("https://github.com/a/b"));
        assert_eq!(view.description.as_deref(), Some("desc"));
    }

    #[test]
    fn project_view_serializes_timestamps_as_rfc3339() {
        let view = project_view(&sample_project(None, None));
        assert_eq!(view.created_at, "2026-01-02T03:04:05+00:00");
        assert_eq!(view.updated_at, "2026-01-02T04:05:06.123456+00:00");
    }

    #[test]
    fn project_view_keeps_missing_optional_fields_as_none() {
        let view = project_view(&sample_project(None, None));
        assert_eq!(view.repo_url, None);
        assert_eq!(view.description, None);
    }
}
