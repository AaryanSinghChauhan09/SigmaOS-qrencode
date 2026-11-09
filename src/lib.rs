// SPDX-License-Identifier: MIT
// SigmaOS-qrencode - QR Code Generator component
// qrencode-style QR code generation for terminal/graphics,
// inspired by Arch's qrencode package.

use std::fmt;

// ============================================================================
// QR CODE STRUCTURE
// ============================================================================

/// QR code error correction levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCorrection {
    /// ~7% recovery
    Low,
    /// ~15% recovery
    Medium,
    /// ~25% recovery (default)
    Quartile,
    /// ~30% recovery
    High,
}

impl ErrorCorrection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "L",
            Self::Medium => "M",
            Self::Quartile => "Q",
            Self::High => "H",
        }
    }
}

/// QR code versions (1-40, determines size)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QrVersion(pub u8);

impl QrVersion {
    /// Modules per side for a version
    pub fn size(&self) -> usize {
        17 + (self.0 as usize * 4)
    }
}

/// QR code symbol
#[derive(Debug, Clone)]
pub struct QrCode {
    /// Version (1-40)
    pub version: QrVersion,
    /// Error correction level
    pub error_correction: ErrorCorrection,
    /// Modules matrix (true = dark)
    pub modules: Vec<Vec<bool>>,
    /// Original data
    pub data: String,
}

impl QrCode {
    /// Create a QR code from text
    pub fn new(data: &str) -> Self {
        Self::with_options(data, ErrorCorrection::Quartile, QrVersion(1))
    }

    /// Create with specific options
    pub fn with_options(data: &str, ec: ErrorCorrection, version: QrVersion) -> Self {
        let size = version.size();
        let mut modules = vec![vec![false; size]; size];

        // Place finder patterns (top-left, top-right, bottom-left)
        place_finder(&mut modules, 0, 0, size);
        place_finder(&mut modules, size - 7, 0, size);
        place_finder(&mut modules, 0, size - 7, size);

        // Place timing patterns
        place_timing(&mut modules, size);

        // Place alignment patterns (for versions >= 2)
        if version.0 >= 2 {
            place_alignment(&mut modules, size);
        }

        // Reserve format info areas (simplified)
        reserve_format_info(&mut modules, size);

        // Fill data area with pseudo-random pattern (simplified)
        // In real impl, this would encode actual data with Reed-Solomon ECC
        fill_data(&mut modules, size, data.as_bytes());

        Self {
            version,
            error_correction: ec,
            modules,
            data: data.to_string(),
        }
    }

    /// Render as ASCII art
    pub fn to_ascii(&self) -> String {
        let mut out = String::new();
        let border = "  ";
        // Top border
        out.push_str(border);
        for _ in 0..self.modules.len() {
            out.push_str("██");
        }
        out.push('\n');

        for row in &self.modules {
            out.push_str(border);
            for &cell in row {
                out.push_str(if cell { "██" } else { "  " });
            }
            out.push('\n');
        }

        // Bottom border
        out.push_str(border);
        for _ in 0..self.modules.len() {
            out.push_str("██");
        }
        out.push('\n');
        out
    }

    /// Render as Unicode block characters
    pub fn to_unicode(&self) -> String {
        let mut out = String::new();
        for row in &self.modules {
            for &cell in row {
                out.push(if cell { '█' } else { ' ' });
            }
            out.push('\n');
        }
        out
    }

    /// Render as matrix of 0/1
    pub fn to_matrix(&self) -> String {
        let mut out = String::new();
        for row in &self.modules {
            for &cell in row {
                out.push(if cell { '1' } else { '0' });
            }
            out.push('\n');
        }
        out
    }

    /// Get module count
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    /// Check if a module is dark
    pub fn is_dark(&self, x: usize, y: usize) -> bool {
        if x >= self.modules.len() || y >= self.modules.len() {
            return false;
        }
        self.modules[y][x]
    }
}

// ============================================================================
// QR CODE HELPERS
// ============================================================================

/// Place finder pattern (7x7) at position
fn place_finder(modules: &mut Vec<Vec<bool>>, x: usize, y: usize, size: usize) {
    for dy in 0..7 {
        for dx in 0..7 {
            let px = x + dx;
            let py = y + dy;
            if px >= size || py >= size { continue; }
            // Finder pattern: outer ring dark, inner ring light, center dark
            let is_outer = dx == 0 || dx == 6 || dy == 0 || dy == 6;
            let is_inner = dx >= 2 && dx <= 4 && dy >= 2 && dy <= 4;
            modules[py][px] = is_outer || is_inner;
        }
    }
}

/// Place timing patterns (alternating line)
fn place_timing(modules: &mut Vec<Vec<bool>>, size: usize) {
    for i in 8..size - 8 {
        // Horizontal timing (row 6)
        modules[6][i] = i % 2 == 0;
        // Vertical timing (col 6)
        modules[i][6] = i % 2 == 0;
    }
}

/// Place alignment patterns (for version >= 2)
fn place_alignment(modules: &mut Vec<Vec<bool>>, size: usize) {
    // Simplified: place one alignment pattern at center for small versions
    if size < 25 { return; }
    let pos = size - 7;
    // 5x5 alignment pattern
    for dy in 0..5 {
        for dx in 0..5 {
            let px = pos + dx;
            let py = pos + dy;
            if px >= size || py >= size { continue; }
            let is_outer = dx == 0 || dx == 4 || dy == 0 || dy == 4;
            let is_center = dx == 2 && dy == 2;
            modules[py][px] = is_outer || is_center;
        }
    }
}

/// Reserve format information areas
fn reserve_format_info(modules: &mut Vec<Vec<bool>>, size: usize) {
    // Format info is stored around finder patterns
    // Top-left: rows 0-8, cols 0-8 (around finder)
    // Top-right: row 8, cols size-8 to size-1
    // Bottom-left: rows size-7 to size-1, col 8
    // Simplified: just mark as reserved (false)
    for i in 0..9 {
        if i < size {
            modules[8][i] = false;
            modules[i][8] = false;
        }
    }
    for i in 0..8 {
        if size - 1 - i < size {
            modules[8][size - 1 - i] = false;
            modules[size - 1 - i][8] = false;
        }
    }
}

/// Fill data area with pseudo-random pattern based on input
fn fill_data(modules: &mut Vec<Vec<bool>>, size: usize, data: &[u8]) {
    // Simple deterministic fill: use data bytes as seed for pattern
    let mut seed: u32 = 0x12345678;
    for &b in data {
        seed = seed.wrapping_mul(1103515245).wrapping_add(b as u32);
    }

    for y in 0..size {
        for x in 0..size {
            // Skip finder/timing/alignment areas (already set)
            if is_function_pattern(x, y, size) { continue; }
            // Simple LCG for pseudo-random fill
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            modules[y][x] = (seed >> 16) & 1 == 1;
        }
    }
}

/// Whether position is a function pattern (finder/timing/alignment/format)
fn is_function_pattern(x: usize, y: usize, size: usize) -> bool {
    // Finder patterns (7x7 at corners + separator)
    let in_tl = x < 9 && y < 9;
    let in_tr = x >= size - 8 && y < 9;
    let in_bl = x < 9 && y >= size - 8;
    if in_tl || in_tr || in_bl { return true; }

    // Timing patterns
    if y == 6 || x == 6 { return true; }

    // Format info areas
    if (y == 8 && x < 9) || (x == 8 && y < 9) { return true; }
    if y == 8 && x >= size - 8 { return true; }
    if x == 8 && y >= size - 7 { return true; }

    // Alignment pattern (bottom-right area for v2+)
    if size >= 25 {
        let pos = size - 7;
        if x >= pos - 1 && x <= pos + 5 && y >= pos - 1 && y <= pos + 5 {
            return true;
        }
    }

    false
}

// ============================================================================
// QR CODE UTILITIES
// ============================================================================

/// Estimate QR version needed for data length
pub fn estimate_version(data_len: usize, ec: ErrorCorrection) -> QrVersion {
    // Simplified capacity table (bytes per version for EC level)
    // Real capacities vary; this is approximate
    let capacity = match ec {
        ErrorCorrection::Low => 170,
        ErrorCorrection::Medium => 130,
        ErrorCorrection::Quartile => 100,
        ErrorCorrection::High => 80,
    };
    let version = (data_len / capacity) + 1;
    QrVersion(version.min(40) as u8)
}

/// Validate QR code data
pub fn validate_data(data: &str) -> Result<(), QrError> {
    if data.is_empty() {
        return Err(QrError::EmptyData);
    }
    if data.len() > 4000 {
        return Err(QrError::DataTooLong(data.len()));
    }
    if data.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err(QrError::InvalidChars);
    }
    Ok(())
}

/// Generate WiFi QR code string (WIFI:T:WPA;S:ssid;P:password;;)
pub fn wifi_qr(ssid: &str, password: &str, encryption: &str) -> String {
    format!("WIFI:T:{};S:{};P:{};;", encryption, ssid, password)
}

/// Generate contact QR code (vCard format)
pub fn contact_qr(name: &str, phone: &str, email: &str) -> String {
    format!("BEGIN:VCARD\nVERSION:3.0\nFN:{}\nTEL:{}\nEMAIL:{}\nEND:VCARD",
        name, phone, email)
}

/// Generate URL QR code
pub fn url_qr(url: &str) -> String {
    url.to_string()
}

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Debug)]
pub enum QrError {
    EmptyData,
    DataTooLong(usize),
    InvalidChars,
}

impl fmt::Display for QrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyData => write!(f, "QR code data is empty"),
            Self::DataTooLong(n) => write!(f, "data too long for QR code: {} bytes", n),
            Self::InvalidChars => write!(f, "invalid characters in QR data"),
        }
    }
}
impl std::error::Error for QrError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_version_size() {
        assert_eq!(QrVersion(1).size(), 21);
        assert_eq!(QrVersion(2).size(), 25);
        assert_eq!(QrVersion(10).size(), 57);
        assert_eq!(QrVersion(40).size(), 177);
    }

    #[test]
    fn test_qr_creation() {
        let qr = QrCode::new("https://example.com");
        assert_eq!(qr.module_count(), 21); // version 1
        assert_eq!(qr.data, "https://example.com");
    }

    #[test]
    fn test_finder_patterns() {
        let qr = QrCode::new("test");
        // Top-left finder: center should be dark
        assert!(qr.is_dark(3, 3));
        // Top-right finder
        assert!(qr.is_dark(qr.module_count() - 4, 3));
        // Bottom-left finder
        assert!(qr.is_dark(3, qr.module_count() - 4));
    }

    #[test]
    fn test_ascii_render() {
        let qr = QrCode::new("hello");
        let ascii = qr.to_ascii();
        assert!(ascii.contains("██"));
        assert!(ascii.len() > 100);
    }

    #[test]
    fn test_unicode_render() {
        let qr = QrCode::new("hello");
        let unicode = qr.to_unicode();
        assert!(unicode.contains('█'));
        assert!(unicode.contains(' '));
    }

    #[test]
    fn test_matrix_render() {
        let qr = QrCode::new("hello");
        let matrix = qr.to_matrix();
        assert!(matrix.contains('1'));
        assert!(matrix.contains('0'));
        assert_eq!(matrix.lines().count(), 21);
    }

    #[test]
    fn test_error_correction() {
        let qr = QrCode::with_options("test", ErrorCorrection::High, QrVersion(1));
        assert_eq!(qr.error_correction, ErrorCorrection::High);
        assert_eq!(qr.error_correction.as_str(), "H");
    }

    #[test]
    fn test_validate_data() {
        assert!(validate_data("hello").is_ok());
        assert!(validate_data("").is_err());
        assert!(validate_data(&"x".repeat(5000)).is_err());
    }

    #[test]
    fn test_wifi_qr() {
        let data = wifi_qr("MyNetwork", "password123", "WPA");
        assert!(data.contains("WIFI:T:WPA"));
        assert!(data.contains("S:MyNetwork"));
        assert!(data.contains("P:password123"));
    }

    #[test]
    fn test_contact_qr() {
        let data = contact_qr("John Doe", "+1234567890", "john@example.com");
        assert!(data.contains("BEGIN:VCARD"));
        assert!(data.contains("FN:John Doe"));
        assert!(data.contains("TEL:+1234567890"));
    }

    #[test]
    fn test_estimate_version() {
        assert_eq!(estimate_version(50, ErrorCorrection::Quartile).0, 1);
        assert_eq!(estimate_version(200, ErrorCorrection::Quartile).0, 3);
        assert_eq!(estimate_version(5000, ErrorCorrection::Quartile).0, 40); // capped
    }
}
