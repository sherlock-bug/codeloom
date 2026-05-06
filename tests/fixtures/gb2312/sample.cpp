// 中文注释 - GB2312 编码测试
// 初始化系统配置

#include <iostream>
#include <string>

/**
 * 设置系统参数
 * @param name 参数名称
 * @param value 参数值
 */
void setup(const char* name, int value) {
    // 打印配置信息
    std::cout << "配置: " << name << " = " << value << std::endl;
}

int main() {
    // 调用初始化函数
    setup("超时时间", 30);
    return 0;
}
