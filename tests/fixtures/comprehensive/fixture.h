// ============================================================
// CodeLoom 规格覆盖测试库
// 基于规格设计，不考虑当前实现偏差
// 覆盖：15 种节点类型 + 11 种边类型
// ============================================================
// 规格说明：
//   节点：function, method, class, struct, enum, enum_value,
//         global, static_var, variable, field, string_literal,
//         macro, template_function, template_class, template_struct
//   边：  calls, overrides, inherits, contains, uses, param_type,
//         return_type, includes, aliases, field_type, template_use
//   工具：list_repos, list_branches, list_symbols, search,
//         semantic_search, inspect, neighbor_graph, call_graph,
//         impact_analysis, path_analysis, inheritance_tree, schema
// ============================================================

#ifndef CODELOOM_SPEC_COVERAGE_H
#define CODELOOM_SPEC_COVERAGE_H

#include <cstddef>

// ===== 宏 (macro) =====
#define SPEC_VERSION 1
#define BUFFER_SIZE 256
#define FORCE_INLINE inline

// ===== 前向声明 (用于测试前向声明过滤) =====
class Widget;
struct Config;

// ===== 枚举 (enum → enum_value via contains) =====
enum LogLevel {
    LOG_DEBUG = 0,
    LOG_INFO = 1,
    LOG_WARN = 2,
    LOG_ERROR = 3
};

enum class Permission : unsigned {
    READ = 1,
    WRITE = 2,
    EXECUTE = 4,
    ALL = 7
};

// ===== 结构体 (struct) =====
struct Point {
    int x;       // field
    int y;       // field
};

// ===== 模板结构体 (template_struct) =====
template <typename T>
struct Holder {
    T value;       // field with template type → template_use
    Holder* next;  // self-ref pointer
};

// ===== 基类 (class) =====
class Base {
public:
    Base();
    virtual ~Base();

    virtual int compute(int x) const;    // method, returns int
    virtual void tick() = 0;             // pure virtual → overrides
    const char* label() const;           // method, returns const char*

    static int instance_count();         // static method

private:
    int value_;                           // field
    static int count_;                    // static_var
};

// ===== 单继承子类 (inherits) =====
class Derived : public Base {
public:
    Derived();
    ~Derived() override;

    int compute(int x) const override;   // overrides
    void tick() override;                // overrides

private:
    Point pt_;                           // field with struct type → field_type
};

// ===== 多重继承 =====
// 3 个基类 → 子类继承全部 3 个 → inherits × 3
class Renderable {
public:
    virtual void draw() = 0;
    virtual int z_order() const;
};

class Clickable {
public:
    virtual bool hit_test(int x, int y) const;
};

class Focusable {
public:
    virtual void focus();
    virtual void blur();
};

class Button : public Renderable, public Clickable, public Focusable {
public:
    void draw() override;
    bool hit_test(int x, int y) const override;
    void focus() override;
};

// ===== 模板类 (template_class) =====
template <typename K, typename V>
class Map {
public:
    void insert(const K& key, const V& val);   // param_type → template_use
    V find(const K& key) const;
    int size() const;
    bool empty() const;
    void clear();

private:
    struct Node {
        K key;
        V value;
        Node* next;
    };
    Node* head_;
};

// ===== 别名/typedef (aliases 边) =====
typedef Point* PointPtr;
using IntCallback = int(*)(int);  // 注意：函数指针可能不产生 aliases 边
typedef unsigned int uint32;
using LogCallback = void(*)(LogLevel, const char*);

// ===== 全局变量 (global) =====
extern int g_verbosity;
extern const char* g_app_name;

// ===== 模板函数 (template_function) =====
template <typename T>
T max_val(T a, T b) {
    return (a > b) ? a : b;
}

template <typename T>
void exchange(T& a, T& b) {
    T tmp = a;
    a = b;
    b = tmp;
}

// ===== 自由函数 (function) =====
int multiply(int a, int b);
double divide(double a, double b);
void report(LogLevel level, const char* msg);   // param_type: LogLevel
int compute_something(double scale);
void flush_all();

// ===== 内联函数 =====
FORCE_INLINE int clamp(int v, int lo, int hi) {
    if (v < lo) return lo;
    if (v > hi) return hi;
    return v;
}

#endif // CODELOOM_SPEC_COVERAGE_H
