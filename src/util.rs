fn hex_pair_to_byte(hex: &str) -> Result<f64, String> {
    let mut value = 0;
    for c in hex.bytes() {
        value <<= 4;
        value |= match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => return Err(format!("Invalid hex char: {}", c)),
        };
    }
    let value = value as f64;
    Ok(value / 255.0)
}
// cairo 颜色 0 - 1
pub fn hex_to_argb(color: &str) -> Result<(f64, f64, f64, f64), String> {
    // 验证长度
    let has_alpha = match color.len() {
        7 => false,
        9 => true,
        _ => return Err("Invalid length".to_string()),
    };

    // 验证格式
    if !color.starts_with('#') {
        return Err("Must start with #".to_string());
    }

    // 验证字符有效性
    for c in color[1..].bytes() {
        if !c.is_ascii_hexdigit() {
            return Err(format!("Invalid hex character: {}", c as char));
        }
    }

    // 解析颜色分量
    let parse = |start, end| -> Result<f64, String> { hex_pair_to_byte(&color[start..end]) };

    let (a, r, g, b) = if has_alpha {
        (parse(1, 3)?, parse(3, 5)?, parse(5, 7)?, parse(7, 9)?)
    } else {
        (1.0, parse(1, 3)?, parse(3, 5)?, parse(5, 7)?)
    };

    Ok((a, r, g, b))
}
