use std::io::Read;

use dmg_background_core::{
    BACKGROUND_1X_PATH, BACKGROUND_2X_PATH, CoreError, LAYOUT_PATH, LayoutDocument, PREVIEW_PATH,
    README_PATH, Window, export_contract_zip,
};

fn png_fixture(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("PNG header should encode");
        let pixel_count = usize::try_from(width)
            .expect("fixture width should fit usize")
            .checked_mul(usize::try_from(height).expect("fixture height should fit usize"))
            .expect("fixture pixel count should fit usize");
        writer
            .write_image_data(&vec![0; pixel_count])
            .expect("PNG pixels should encode");
    }
    bytes
}

fn png_with_invalid_deflate_stream(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = png_fixture(width, height);
    let mut offset = 8;

    while offset + 12 <= bytes.len() {
        let length = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("chunk length should contain four bytes"),
        ) as usize;
        let chunk_type_start = offset + 4;
        let data_start = offset + 8;
        let data_end = data_start + length;
        let crc_end = data_end + 4;
        assert!(crc_end <= bytes.len(), "fixture chunk should be in bounds");

        if &bytes[chunk_type_start..data_start] == b"IDAT" {
            assert!(length > 0, "fixture IDAT should contain compressed data");
            bytes[data_start] = 0;

            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&bytes[chunk_type_start..data_end]);
            bytes[data_end..crc_end].copy_from_slice(&hasher.finalize().to_be_bytes());
            return bytes;
        }

        offset = crc_end;
    }

    panic!("fixture should contain an IDAT chunk");
}

fn png_with_invalid_adler_checksum(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = png_fixture(width, height);
    let mut offset = 8;
    let mut final_idat = None;

    while offset + 12 <= bytes.len() {
        let length = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("chunk length should contain four bytes"),
        ) as usize;
        let chunk_type_start = offset + 4;
        let data_start = offset + 8;
        let data_end = data_start + length;
        let crc_end = data_end + 4;
        assert!(crc_end <= bytes.len(), "fixture chunk should be in bounds");

        if &bytes[chunk_type_start..data_start] == b"IDAT" {
            final_idat = Some((chunk_type_start, data_start, data_end, crc_end));
        }

        offset = crc_end;
    }

    let (chunk_type_start, data_start, data_end, crc_end) =
        final_idat.expect("fixture should contain an IDAT chunk");
    assert!(
        data_end - data_start >= 4,
        "final IDAT should contain the Adler-32 checksum"
    );
    bytes[data_end - 1] ^= 0xff;

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&bytes[chunk_type_start..data_end]);
    bytes[data_end..crc_end].copy_from_slice(&hasher.finalize().to_be_bytes());
    bytes
}

fn png_with_corrupt_idat_crc(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = png_fixture(width, height);
    let mut offset = 8;

    while offset + 12 <= bytes.len() {
        let length = u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("chunk length should contain four bytes"),
        ) as usize;
        let data_start = offset + 8;
        let data_end = data_start + length;
        let crc_end = data_end + 4;
        assert!(crc_end <= bytes.len(), "fixture chunk should be in bounds");

        if &bytes[offset + 4..data_start] == b"IDAT" {
            bytes[data_end] ^= 0xff;
            return bytes;
        }

        offset = crc_end;
    }

    panic!("fixture should contain an IDAT chunk");
}

fn apng_fixture(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .set_animated(1, 0)
            .expect("APNG control chunks should encode");
        let mut writer = encoder.write_header().expect("APNG header should encode");
        let pixel_count = usize::try_from(width)
            .expect("fixture width should fit usize")
            .checked_mul(usize::try_from(height).expect("fixture height should fit usize"))
            .expect("fixture pixel count should fit usize");
        writer
            .write_image_data(&vec![0; pixel_count])
            .expect("APNG pixels should encode");
    }
    bytes
}

#[test]
fn exported_zip_contains_the_five_frozen_contract_members() {
    let document = LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Example",
    )
    .expect("fixture window should be valid");
    let one_x = png_fixture(800, 450);
    let two_x = png_fixture(1600, 900);
    let preview = png_fixture(800, 450);
    let zip_bytes = export_contract_zip(&document, &one_x, &two_x, &preview)
        .expect("contract ZIP should export");

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes))
        .expect("export should be a readable ZIP");
    assert_eq!(archive.len(), 5);

    for (path, expected) in [
        (BACKGROUND_1X_PATH, one_x.as_slice()),
        (BACKGROUND_2X_PATH, two_x.as_slice()),
        (PREVIEW_PATH, preview.as_slice()),
    ] {
        let mut file = archive.by_name(path).expect("member should exist");
        let mut content = Vec::new();
        file.read_to_end(&mut content).expect("member should read");
        assert_eq!(content, expected, "member {path} content should round-trip");
    }

    let mut layout = String::new();
    archive
        .by_name(LAYOUT_PATH)
        .expect("layout member should exist")
        .read_to_string(&mut layout)
        .expect("layout should be UTF-8");
    let decoded: LayoutDocument = serde_json::from_str(&layout).expect("layout JSON should parse");
    decoded.validate().expect("layout JSON should validate");

    let mut readme = String::new();
    archive
        .by_name(README_PATH)
        .expect("README member should exist")
        .read_to_string(&mut readme)
        .expect("README should be UTF-8");
    assert!(readme.contains("dmg_background export contract v1"));
}

#[test]
fn export_rejects_invalid_png_assets() {
    let document = LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Example",
    )
    .expect("fixture window should be valid");
    let two_x = png_fixture(1600, 900);
    let preview = png_fixture(800, 450);

    assert_eq!(
        export_contract_zip(&document, b"not-a-png", &two_x, &preview),
        Err(CoreError::InvalidPngAsset {
            asset: BACKGROUND_1X_PATH,
        })
    );
}

#[test]
fn export_rejects_png_assets_with_a_corrupt_iend_crc() {
    let document = LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Example",
    )
    .expect("fixture window should be valid");
    let mut corrupt_one_x = png_fixture(800, 450);
    *corrupt_one_x
        .last_mut()
        .expect("fixture should not be empty") ^= 0xff;
    let two_x = png_fixture(1600, 900);
    let preview = png_fixture(800, 450);

    assert_eq!(
        export_contract_zip(&document, &corrupt_one_x, &two_x, &preview),
        Err(CoreError::InvalidPngAsset {
            asset: BACKGROUND_1X_PATH,
        })
    );
}

#[test]
fn export_rejects_png_assets_with_a_corrupt_idat_crc() {
    let document = LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Example",
    )
    .expect("fixture window should be valid");
    let corrupt_one_x = png_with_corrupt_idat_crc(800, 450);
    let two_x = png_fixture(1600, 900);
    let preview = png_fixture(800, 450);

    assert_eq!(
        export_contract_zip(&document, &corrupt_one_x, &two_x, &preview),
        Err(CoreError::InvalidPngAsset {
            asset: BACKGROUND_1X_PATH,
        })
    );
}

#[test]
fn export_rejects_png_with_valid_chunk_crc_but_invalid_deflate_stream() {
    let document = LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Example",
    )
    .expect("fixture window should be valid");
    let invalid_one_x = png_with_invalid_deflate_stream(800, 450);
    let two_x = png_fixture(1600, 900);
    let preview = png_fixture(800, 450);

    assert_eq!(
        export_contract_zip(&document, &invalid_one_x, &two_x, &preview),
        Err(CoreError::InvalidPngAsset {
            asset: BACKGROUND_1X_PATH,
        })
    );
}

#[test]
fn export_rejects_png_with_valid_chunk_crc_but_invalid_adler_checksum() {
    let document = LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Example",
    )
    .expect("fixture window should be valid");
    let invalid_one_x = png_with_invalid_adler_checksum(800, 450);
    let two_x = png_fixture(1600, 900);
    let preview = png_fixture(800, 450);

    assert_eq!(
        export_contract_zip(&document, &invalid_one_x, &two_x, &preview),
        Err(CoreError::InvalidPngAsset {
            asset: BACKGROUND_1X_PATH,
        })
    );
}

#[test]
fn export_rejects_animated_png_assets() {
    let document = LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Example",
    )
    .expect("fixture window should be valid");
    let animated_one_x = apng_fixture(800, 450);
    let two_x = png_fixture(1600, 900);
    let preview = png_fixture(800, 450);

    assert_eq!(
        export_contract_zip(&document, &animated_one_x, &two_x, &preview),
        Err(CoreError::InvalidPngAsset {
            asset: BACKGROUND_1X_PATH,
        })
    );
}

#[test]
fn export_rejects_png_dimensions_that_do_not_match_the_layout() {
    let document = LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Example",
    )
    .expect("fixture window should be valid");
    let wrong_one_x = png_fixture(801, 450);
    let two_x = png_fixture(1600, 900);
    let preview = png_fixture(800, 450);

    assert_eq!(
        export_contract_zip(&document, &wrong_one_x, &two_x, &preview),
        Err(CoreError::PngDimensionMismatch {
            asset: BACKGROUND_1X_PATH,
            expected_width: 800,
            expected_height: 450,
            actual_width: 801,
            actual_height: 450,
        })
    );
}
