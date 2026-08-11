/// Formats a byte count as a human-readable size (e.g. `1.44 KB`, `2.10 MB`).
pub fn format_bytes(len: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let len = len as f64;
    if len < KB {
        format!("{len} B")
    } else if len < MB {
        format!("{:.2} KB", len / KB)
    } else if len < GB {
        format!("{:.2} MB", len / MB)
    } else {
        format!("{:.2} GB", len / GB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bytes_under_a_kilobyte_with_no_decimals() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn formats_kilobytes() {
        assert_eq!(format_bytes(1478), "1.44 KB");
        assert_eq!(format_bytes(1024), "1.00 KB");
    }

    #[test]
    fn formats_megabytes() {
        assert_eq!(format_bytes(1_474_560), "1.41 MB");
    }

    #[test]
    fn formats_gigabytes() {
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2.00 GB");
    }
}
