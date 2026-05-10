# gb2312-encoding-support

## Purpose
CodeLoom gb2312-encoding-support 功能域。本规范描述此功能的需求和行为。

## Purpose
CodeLoom 源码文件编码自动检测与转码功能。读取源文件时先尝试 UTF-8 解码，失败则用 GB18030（兼容 GBK/GB2312）解码，确保中文编码的 C++/Python/Java 等源码能正常索引和解析。

## Requirements

### Requirement: 源码文件编码自动检测
系统 SHALL 在读取源码文件时先尝试 UTF-8 解码，失败则用 GB18030 解码（兼容 GBK/GB2312）。

#### Scenario: UTF-8 文件正常读取
- GIVEN 源码文件使用 UTF-8 编码
- WHEN 调用 read_file_smart 读取
- THEN 返回正确的 UTF-8 字符串内容

#### Scenario: GB2312 文件自动转码
- GIVEN 源码文件使用 GB2312 编码，包含中文字符
- WHEN 调用 read_file_smart 读取
- THEN 自动检测并转为 UTF-8 字符串
- AND 中文字符正确保留

#### Scenario: 无效字节宽容处理
- GIVEN 文件包含既非 UTF-8 也非 GB18030 的字节序列
- WHEN 调用 read_file_smart 读取
- THEN 使用 UTF-8 替换字符（U+FFFD）替换无效字节
- AND 不 panic 或返回错误

### Requirement: 索引流程编码兼容
parse_file、index_includes、index_docs SHALL 使用 read_file_smart 替代 std::fs::read_to_string。

#### Scenario: GB2312 源码正常索引
- GIVEN 一个使用 GB2312 编码的 C++ 源文件
- WHEN 对该文件执行 codeloom index
- THEN 正常提取符号和 #include 关系
- AND 不报编码错误

#### Scenario: 混合编码仓库正常索引
- GIVEN 一个仓库中同时存在 UTF-8 和 GB2312 编码的源文件
- WHEN 执行 codeloom index
- THEN 两种编码的文件均正常解析
- AND 符号总数等于两种文件之和
