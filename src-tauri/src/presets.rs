use crate::types::StripOptions;

pub fn max_privacy() -> StripOptions {
    StripOptions {
        preset: "max_privacy".into(),
        strip_gps: true,
        strip_author: true,
        strip_camera: true,
        strip_timestamps: true,
        strip_icc: true,
        strip_iptc: true,
        strip_xmp: true,
        strip_makernotes: true,
        strip_thumbnail: true,
        inject_neutral: false,
        pdf_full_rewrite: true,
        ..Default::default()
    }
}

pub fn keep_quality() -> StripOptions {
    StripOptions {
        preset: "keep_quality".into(),
        strip_gps: true,
        strip_author: true,
        strip_camera: false,
        strip_timestamps: false,
        strip_icc: false,
        strip_iptc: true,
        strip_xmp: true,
        strip_makernotes: true,
        strip_thumbnail: true,
        inject_neutral: false,
        pdf_full_rewrite: false,
        ..Default::default()
    }
}

pub fn social_media() -> StripOptions {
    StripOptions {
        preset: "social_media".into(),
        strip_gps: true,
        strip_author: true,
        strip_camera: false,
        strip_timestamps: true,
        strip_icc: false,
        strip_iptc: true,
        strip_xmp: true,
        strip_makernotes: true,
        strip_thumbnail: true,
        inject_neutral: false,
        pdf_full_rewrite: false,
        ..Default::default()
    }
}

pub fn documents_only() -> StripOptions {
    StripOptions {
        preset: "documents_only".into(),
        strip_gps: true,
        strip_author: true,
        strip_camera: false,
        strip_timestamps: true,
        strip_icc: false,
        strip_iptc: false,
        strip_xmp: true,
        strip_makernotes: false,
        strip_thumbnail: false,
        inject_neutral: false,
        pdf_full_rewrite: true,
        ..Default::default()
    }
}

pub fn from_id(id: &str) -> StripOptions {
    match id {
        "max_privacy" => max_privacy(),
        "keep_quality" => keep_quality(),
        "social_media" => social_media(),
        "documents_only" => documents_only(),
        _ => max_privacy(),
    }
}
