use bevy::prelude::*;
use serde::Deserialize;

const DIGITS_JSON: &str = include_str!("../../../../assets/fonts/fredoka/fredoka-bold-digits.json");

#[derive(Debug, Deserialize)]
struct AtlasJson {
    atlas: AtlasInfo,
    glyphs: Vec<GlyphInfo>,
}

#[derive(Debug, Deserialize)]
struct AtlasInfo {
    width: f32,
    height: f32,
}

#[derive(Debug, Deserialize)]
struct GlyphInfo {
    unicode: u32,
    #[serde(rename = "atlasBounds")]
    atlas_bounds: Bounds,
}

#[derive(Debug, Deserialize)]
struct Bounds {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
}

/// Parsed digit atlas with UV coordinates for each digit 0-8
#[derive(Resource, Clone)]
pub struct DigitAtlas {
    pub texture: Handle<Image>,
    /// UV bounds for each digit: [u_min, v_min, u_max, v_max]
    pub digit_uvs: [[f32; 4]; 9],
}

impl DigitAtlas {
    /// Parse the embedded JSON and create the atlas resource
    pub fn load(asset_server: &AssetServer) -> Self {
        // Load the texture
        let texture = asset_server.load("fonts/fredoka/fredoka-bold-digits.png");

        // Parse the embedded JSON
        let atlas_data: AtlasJson =
            serde_json::from_str(DIGITS_JSON).expect("Failed to parse embedded digits.json");

        let atlas_width = atlas_data.atlas.width;
        let atlas_height = atlas_data.atlas.height;

        // Convert atlas bounds to UV coordinates
        let mut digit_uvs = [[0.0; 4]; 9];

        for glyph in atlas_data.glyphs {
            let digit_idx = (glyph.unicode - 48) as usize; // '0' is unicode 48

            if digit_idx < 9 {
                let bounds = &glyph.atlas_bounds;
                // NOTE: JSON has yOrigin="bottom", but WebGPU uses top-left origin
                // So we need to flip V coordinates: v_flipped = 1.0 - v_original
                digit_uvs[digit_idx] = [
                    bounds.left / atlas_width,            // u_min
                    1.0 - (bounds.top / atlas_height),    // v_min (flipped)
                    bounds.right / atlas_width,           // u_max
                    1.0 - (bounds.bottom / atlas_height), // v_max (flipped)
                ];
            }
        }

        Self { texture, digit_uvs }
    }

    /// Convert to shader-compatible format (Vec4 array)
    pub fn to_shader_uvs(&self) -> [Vec4; 9] {
        self.digit_uvs
            .map(|uv| Vec4::new(uv[0], uv[1], uv[2], uv[3]))
    }
}
