# Delta for Code Indexing

## ADDED Requirements

### Requirement: 解析多种编程语言的源代码
系统 SHALL 使用 tree-sitter 解析器解析 C++、Python、Java、TypeScript、Go 的源代码文件，提取符号定义（函数、类、方法、枚举、结构体、类型别名、变量、宏）。

#### Scenario: 解析 C++ 源文件
- GIVEN 一个包含函数、类、枚举定义的 C++ 源文件
- WHEN 用户执行索引命令
- THEN 系统提取所有函数名、类名、方法名、枚举值、结构体名、typedef 别名
- AND 每个符号包含文件名、起始行号、结束行号、完整定义文本

#### Scenario: 解析 Python 源文件
- GIVEN 一个包含类和方法定义的 Python 源文件
- WHEN 用户执行索引命令
- THEN 系统提取类名、方法名、函数名及其所属类
- AND 每个符号包含文件路径和行号范围

### Requirement: 构建调用图关系
系统 SHALL 在解析源码时提取函数和方法之间的调用关系，形成调用图边。

#### Scenario: 提取函数间调用关系
- GIVEN 源文件 func_a 的调用表达式中引用了 func_b
- WHEN 系统完成符号提取
- THEN 在边表中创建 calls 类型的边连接 func_a 和 func_b
- AND func_a 为 source，func_b 为 target

### Requirement: 构建继承关系
系统 SHALL 提取类和接口之间的继承与实现关系。

#### Scenario: C++ 类继承
- GIVEN class B 声明为 class B : public A
- WHEN 系统解析该文件
- THEN 创建 inherits 边连接 B 和 A

#### Scenario: Python 类继承
- GIVEN class Child(Base) 声明
- WHEN 系统解析该文件
- THEN 创建 inherits 边连接 Child 和 Base

### Requirement: 提取模块级依赖关系
系统 SHALL 提取文件之间的 include/import 依赖关系。

#### Scenario: C++ include 依赖
- GIVEN file_a.cpp 包含 #include "file_b.h"
- WHEN 系统解析 file_a.cpp
- THEN 创建 depends 边连接 file_a.cpp 和 file_b.h

### Requirement: 增量索引
系统 SHALL 支持增量索引，仅重新解析自上次索引以来修改过的文件。

#### Scenario: 修改单个文件后增量索引
- GIVEN 项目已完成全量索引，用户修改了单个文件
- WHEN 再次执行索引命令
- THEN 系统仅解析该修改文件并更新其符号
- AND 未修改文件的符号保持不变

### Requirement: 支持百万行级别代码库
系统 SHALL 支持索引超过一百万行的代码库，初次索引时间不超过 5 分钟。

#### Scenario: 索引百万行 C++ 项目
- GIVEN 一个包含 10,000+ 文件、1,000,000+ 行的 C++ 项目
- WHEN 执行全量索引
- THEN 索引在 5 分钟内完成
- AND SQLite 数据库大小不超过 500MB
