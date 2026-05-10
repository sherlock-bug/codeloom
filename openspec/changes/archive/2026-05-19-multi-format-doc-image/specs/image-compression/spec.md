# Delta for image-compression

## ADDED Requirements

### Requirement: 图片压缩存储
系统 SHALL 将提取的图片按需压缩为 WebP 格式，以 BLOB 存入 SQLite doc_images 表。

#### Scenario: 大图压缩
- GIVEN 一张原始字节超过 100KB 或宽度超过 800px 的图片
- WHEN 进行压缩
- THEN 系统 SHALL 输出 max 1920px 宽、quality 80% 的 WebP 字节

#### Scenario: 小图原样保留
- GIVEN 一张原始字节 ≤ 100KB 且宽度 ≤ 800px 的图片
- WHEN 进行压缩
- THEN 系统 SHALL 保持原始格式和字节不变，直接存入 image_data BLOB

#### Scenario: 已压缩格式跳过
- GIVEN 一张已经是 WebP 格式且 ≤ 200KB 的图片
- WHEN 进行压缩
- THEN 系统 SHALL 直接使用原始字节，不重新编码

#### Scenario: 压缩后记录元数据
- GIVEN 一张图片完成处理
- WHEN 写入 doc_images 表
- THEN original_size SHALL 记录原始字节数，compressed_size 记录存储字节数，width/height 记录像素尺寸

#### Scenario: 不落磁盘
- GIVEN 任何图片完成处理
- WHEN 写入 doc_images 表
- THEN 系统 SHALL NOT 创建任何磁盘文件，image_data 直接以 BLOB 形式存入 SQLite
