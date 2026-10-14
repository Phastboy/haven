//! Profile edit route at `/profile/edit`.

use crate::app::components::button::{ButtonVariant, button, button_link};
use crate::app::components::field::{text_field, textarea_field};
use haven_domain::ports::RepoError;
use haven_domain::user::{Bio, DisplayName, ProfileError, ProfileHandle, UserProfileUpdate};
use topcoat::{
    Result as TopcoatResult,
    context::Cx,
    router::error::see_other,
    view::{View, view},
};

#[derive(Default, Clone)]
pub struct ProfileFormFields {
    pub display_name: String,
    pub handle: String,
    pub bio: String,
}

#[derive(Default, Clone)]
pub struct ProfileFieldErrors {
    pub display_name: Option<String>,
    pub handle: Option<String>,
    pub bio: Option<String>,
}

struct ProfileEditFormViewData {
    fields: ProfileFormFields,
    errors: ProfileFieldErrors,
    avatar_url: Option<String>,
    avatar_srcset: Option<String>,
    avatar_initial: String,
    has_avatar: bool,
    notice_banner: Option<String>,
    error_banner: Option<String>,
    status_code: Option<http::StatusCode>,
}

fn user_avatar_data(
    cx: &Cx,
    user: &haven_domain::user::User,
) -> (Option<String>, Option<String>, String, bool) {
    let cloud_name = crate::app::state::cloudinary_cloud_name(cx);
    let (avatar_url, avatar_srcset) = match user.avatar_public_id.as_ref() {
        Some(public_id) => (
            Some(crate::app::components::image::avatar_url(
                cloud_name,
                public_id.as_str(),
            )),
            Some(crate::app::components::image::avatar_srcset(
                cloud_name,
                public_id.as_str(),
            )),
        ),
        None => (None, None),
    };
    let avatar_initial =
        crate::app::components::identity_block::extract_avatar_initial(user.display_name.as_str());
    let has_avatar = user.avatar_public_id.is_some();
    (avatar_url, avatar_srcset, avatar_initial, has_avatar)
}

fn render_profile_edit_form(cx: &Cx, view_data: ProfileEditFormViewData) -> impl View {
    let __cx = cx;
    let fields = view_data.fields;
    let errors = view_data.errors;

    view! {
        <div class="profile-edit-page">
            <meta name="robots" content="noindex" />
            if let Some(status) = view_data.status_code {
                (status)
            }
            <div class="page-header">
                <h1>"Edit Profile"</h1>
                <p class="text-muted">"Update your public name, handle, bio, and avatar."</p>
            </div>
            if let Some(notice) = view_data.notice_banner {
                <div class="notice-banner" role="status">
                    <p>(notice)</p>
                </div>
            }
            if let Some(err) = view_data.error_banner {
                <div class="form-error-banner" role="alert">
                    <p>(err)</p>
                </div>
            }

            <div class="avatar-edit-card">
                <div class="avatar-slot" aria-hidden="true">
                    if let Some(avatar_url) = view_data.avatar_url {
                        if let Some(avatar_srcset) = view_data.avatar_srcset {
                            <img src=(avatar_url) srcset=(avatar_srcset) alt="" class="avatar-img" />
                        } else {
                            <img src=(avatar_url) alt="" class="avatar-img" />
                        }
                    } else {
                        <span class="avatar-initial">(view_data.avatar_initial)</span>
                    }
                </div>
                <div class="avatar-edit-content">
                    <span class="avatar-edit-title">"Profile Photo"</span>
                    <p class="avatar-edit-help text-muted">"JPEG, PNG, or WebP up to 5 MB."</p>
                    <div class="avatar-edit-actions">
                        <form method="post" action="/profile/avatar" enctype="multipart/form-data" class="avatar-upload-form">
                            <input type="file" name="avatar" accept="image/jpeg,image/png,image/webp" required="required" class="avatar-file-input" />
                            <button type="submit" class="btn btn-secondary">"Upload photo"</button>
                        </form>
                        if view_data.has_avatar {
                            <form method="post" action="/profile/avatar/delete" class="avatar-delete-form">
                                <button type="submit" class="btn btn-secondary">"Remove photo"</button>
                            </form>
                        }
                    </div>
                </div>
            </div>

            <form method="post" action="/profile/edit" class="form-stack">
                text_field(
                    label: "Display Name",
                    name: "display_name",
                    value: Some(fields.display_name),
                    required: true,
                    minlength: Some(2),
                    maxlength: Some(50),
                    help: Some("The name people see on your offers.".to_string()),
                    error: errors.display_name,
                )
                text_field(
                    label: "Handle",
                    name: "handle",
                    value: Some(fields.handle),
                    required: true,
                    minlength: Some(3),
                    maxlength: Some(30),
                    help: Some("Unique URL handle for your profile (e.g. alex).".to_string()),
                    error: errors.handle,
                )
                textarea_field(
                    label: "Bio",
                    name: "bio",
                    value: Some(fields.bio),
                    maxlength: Some(500),
                    help: Some("A short bio describing what you make or offer (up to 500 characters).".to_string()),
                    error: errors.bio,
                )
                <div class="form-actions">
                    button(
                        text: "Save Changes",
                        variant: ButtonVariant::Primary,
                    )
                    button_link(
                        href: "/profile",
                        text: "Cancel",
                        variant: ButtonVariant::Secondary,
                    )
                </div>
            </form>
        </div>
    }
}

/// Query parameters on `/profile/edit`.
#[topcoat::router::query_params]
pub struct ProfileEditQuery {
    pub error: Option<String>,
    pub avatar_updated: Option<String>,
    pub avatar_deleted: Option<String>,
}

#[topcoat::router::page]
pub async fn edit_page(cx: &Cx) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_auth(cx).await?;
    let query = topcoat::router::query_params::<ProfileEditQuery>(cx).ok();

    let notice_banner = if query.as_ref().and_then(|q| q.avatar_updated.as_deref()) == Some("1") {
        Some("Avatar photo updated successfully.".to_string())
    } else if query.as_ref().and_then(|q| q.avatar_deleted.as_deref()) == Some("1") {
        Some("Avatar photo removed.".to_string())
    } else {
        None
    };

    let error_banner = match query.as_ref().and_then(|q| q.error.as_deref()) {
        Some("too_large") => {
            Some("Avatar image exceeds the 5 MB limit. Please select a smaller photo.".to_string())
        }
        Some("invalid_format") => Some(
            "Invalid image format. Supported formats: JPEG, PNG, and WebP (SVG is not allowed)."
                .to_string(),
        ),
        Some("empty_file" | "missing_file") => {
            Some("Please select an image file to upload.".to_string())
        }
        Some("upload_failed") => Some("Image upload failed. Please try again.".to_string()),
        _ => None,
    };

    let (avatar_url, avatar_srcset, avatar_initial, has_avatar) = user_avatar_data(cx, &user);

    Ok(render_profile_edit_form(
        cx,
        ProfileEditFormViewData {
            fields: ProfileFormFields {
                display_name: user.display_name.as_str().to_string(),
                handle: user.handle.as_str().to_string(),
                bio: user
                    .bio
                    .as_ref()
                    .map_or_else(String::new, |b| b.as_str().to_string()),
            },
            errors: ProfileFieldErrors::default(),
            avatar_url,
            avatar_srcset,
            avatar_initial,
            has_avatar,
            notice_banner,
            error_banner,
            status_code: None,
        },
    ))
}

#[derive(serde::Deserialize)]
pub struct ProfileForm {
    pub display_name: String,
    pub handle: String,
    pub bio: Option<String>,
}

fn map_display_name_error(err: &ProfileError) -> &'static str {
    match err {
        ProfileError::BlankDisplayName => "Display name cannot be empty.",
        ProfileError::DisplayNameTooShort => "Display name must be at least 2 characters.",
        ProfileError::DisplayNameTooLong => "Display name cannot exceed 50 characters.",
        ProfileError::HandleTooShort
        | ProfileError::HandleTooLong
        | ProfileError::InvalidHandleFormat
        | ProfileError::ReservedHandle(_)
        | ProfileError::BioTooLong => "Invalid display name.",
    }
}

fn map_handle_error(err: &ProfileError) -> &'static str {
    match err {
        ProfileError::HandleTooShort => "Handle must be at least 3 characters.",
        ProfileError::HandleTooLong => "Handle cannot exceed 30 characters.",
        ProfileError::InvalidHandleFormat => {
            "Handle must contain only lowercase letters, digits, and single hyphens."
        }
        ProfileError::ReservedHandle(_) => "This handle is reserved. Please choose another one.",
        ProfileError::BlankDisplayName
        | ProfileError::DisplayNameTooShort
        | ProfileError::DisplayNameTooLong
        | ProfileError::BioTooLong => "Invalid handle.",
    }
}

fn map_bio_error(err: &ProfileError) -> &'static str {
    match err {
        ProfileError::BioTooLong => "Bio cannot exceed 500 characters.",
        ProfileError::BlankDisplayName
        | ProfileError::DisplayNameTooShort
        | ProfileError::DisplayNameTooLong
        | ProfileError::HandleTooShort
        | ProfileError::HandleTooLong
        | ProfileError::InvalidHandleFormat
        | ProfileError::ReservedHandle(_) => "Invalid bio.",
    }
}

fn validate_profile_fields(
    fields: &ProfileFormFields,
) -> Result<UserProfileUpdate, ProfileFieldErrors> {
    let mut errors = ProfileFieldErrors::default();
    let mut has_errors = false;

    let display_name = match DisplayName::parse(&fields.display_name) {
        Ok(name) => Some(name),
        Err(err) => {
            errors.display_name = Some(map_display_name_error(&err).to_string());
            has_errors = true;
            None
        }
    };

    let handle = match ProfileHandle::parse(&fields.handle) {
        Ok(handle) => Some(handle),
        Err(err) => {
            errors.handle = Some(map_handle_error(&err).to_string());
            has_errors = true;
            None
        }
    };

    let bio = if fields.bio.trim().is_empty() {
        None
    } else {
        match Bio::parse(&fields.bio) {
            Ok(b) => Some(b),
            Err(err) => {
                errors.bio = Some(map_bio_error(&err).to_string());
                has_errors = true;
                None
            }
        }
    };

    if has_errors {
        Err(errors)
    } else if let (Some(display_name), Some(handle)) = (display_name, handle) {
        Ok(UserProfileUpdate {
            display_name,
            handle,
            bio,
        })
    } else {
        Err(errors)
    }
}

#[topcoat::router::page(POST)]
pub async fn update_profile(
    cx: &Cx,
    form: Option<topcoat::router::content::Form<ProfileForm>>,
) -> TopcoatResult<impl View> {
    let user = crate::app::auth::guard::require_auth(cx).await?;
    let (avatar_url, avatar_srcset, avatar_initial, has_avatar) = user_avatar_data(cx, &user);

    let Some(form) = form else {
        return Ok(render_profile_edit_form(
            cx,
            ProfileEditFormViewData {
                fields: ProfileFormFields {
                    display_name: user.display_name.as_str().to_string(),
                    handle: user.handle.as_str().to_string(),
                    bio: user
                        .bio
                        .as_ref()
                        .map_or_else(String::new, |b| b.as_str().to_string()),
                },
                errors: ProfileFieldErrors::default(),
                avatar_url,
                avatar_srcset,
                avatar_initial,
                has_avatar,
                notice_banner: None,
                error_banner: Some("Missing form submission.".to_string()),
                status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
            },
        ));
    };

    let f = form.0;
    let fields = ProfileFormFields {
        display_name: f.display_name,
        handle: f.handle,
        bio: f.bio.unwrap_or_default(),
    };

    let update = match validate_profile_fields(&fields) {
        Ok(update) => update,
        Err(field_errors) => {
            return Ok(render_profile_edit_form(
                cx,
                ProfileEditFormViewData {
                    fields,
                    errors: field_errors,
                    avatar_url,
                    avatar_srcset,
                    avatar_initial,
                    has_avatar,
                    notice_banner: None,
                    error_banner: Some("Please correct the errors below.".to_string()),
                    status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
                },
            ));
        }
    };

    let registry = crate::app::state::registry(cx);
    match registry.users().update_profile(user.id, &update).await {
        Ok(_) => Err(see_other("/profile?updated=1").into()),
        Err(RepoError::Conflict { .. }) => {
            let errors = ProfileFieldErrors {
                handle: Some(
                    "This handle is already taken. Please choose another one.".to_string(),
                ),
                ..Default::default()
            };
            Ok(render_profile_edit_form(
                cx,
                ProfileEditFormViewData {
                    fields,
                    errors,
                    avatar_url,
                    avatar_srcset,
                    avatar_initial,
                    has_avatar,
                    notice_banner: None,
                    error_banner: Some("The requested handle is already taken.".to_string()),
                    status_code: Some(http::StatusCode::UNPROCESSABLE_ENTITY),
                },
            ))
        }
        Err(err) => Err(crate::app::state::map_repo_err(err)),
    }
}
