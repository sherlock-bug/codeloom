// 文件读取工具：UTF-8 优先，GB18030 兜底
use std::io;
use std::path::Path;

/// 智能读取文件内容：先尝试 UTF-8，失败则用 GB18030（兼容 GBK/GB2312）解码，最终兜底 lossy
pub fn read_file_smart(path: impl AsRef<Path>) -> io::Result<String> {
    let bytes = std::fs::read(path.as_ref())?;
    Ok(decode_bytes(&bytes))
}

fn decode_bytes(bytes: &[u8]) -> String {
    // 1. 快路径：UTF-8
    if let Ok(s) = String::from_utf8(bytes.to_vec()) {
        return s;
    }
    // 2. GB18030（兼容 GBK/GB2312）
    let (decoded, _, had_errors) = encoding_rs::GB18030.decode(bytes);
    if !had_errors {
        return decoded.into_owned();
    }
    // 3. 兜底：lossy UTF-8
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_utf8_fast_path() {
        let result = decode_bytes("hello world".as_bytes());
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_utf8_chinese() {
        let result = decode_bytes("你好世界".as_bytes());
        assert_eq!(result, "你好世界");
    }

    #[test]
    fn test_gb2312_decoding() {
        // "中文测试" in GB2312/GBK
        let gbk_bytes = vec![0xD6, 0xD0, 0xCE, 0xC4, 0xB2, 0xE2, 0xCA, 0xD4];
        let result = decode_bytes(&gbk_bytes);
        assert_eq!(result, "中文测试");
    }

    #[test]
    fn test_gb2312_with_ascii() {
        // "void test() { // 初始化变量" in GBK
        let mut gbk = b"void test() { // ".to_vec();
        gbk.extend_from_slice(&[0xB3, 0xF5, 0xCA, 0xBC, 0xBB, 0xAF, 0xB1, 0xE4, 0xC1, 0xBF]);
        let result = decode_bytes(&gbk);
        assert_eq!(result, "void test() { // 初始化变量");
    }

    #[test]
    fn test_gb2312_file_roundtrip() {
        let tmp = std::env::temp_dir().join("test_gb2312.cpp");
        let gbk_bytes = vec![
            0x2F, 0x2F, 0x20, 0xD6, 0xD0, 0xCE, 0xC4, 0xD7, 0xA2, 0xCA, 0xCD, 0x0A,  // // 中文注释\n
            0x76, 0x6F, 0x69, 0x64, 0x20, 0x73, 0x65, 0x74, 0x75, 0x70, 0x28, 0x29, 0x20, 0x7B, 0x0A,  // void setup() {\n
            0x7D, 0x0A,  // }\n
        ];
        std::fs::write(&tmp, &gbk_bytes).unwrap();
        let result = read_file_smart(&tmp).unwrap();
        assert!(result.contains("中文注释"));
        assert!(result.contains("void setup()"));
        std::fs::remove_file(&tmp).ok();
    }
}
