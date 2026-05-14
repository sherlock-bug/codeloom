// ============================================================
// CodeLoom 规格覆盖测试库 - 实现文件
// 产生：calls, uses, references, string_literal, includes 等边
// ============================================================
#include "fixture.h"
#include <cstdio>
#include <cstring>

// ===== 全局变量定义 (global) =====
int g_verbosity = 1;
const char* g_app_name = "SpecCoverage";       // global + string_literal

// ===== 静态变量定义 (static_var) =====
int Base::count_ = 0;

// ===== Base 实现 =====
Base::Base() : value_(0) {
    count_++;                                   // references static_var
}

Base::~Base() {
    count_--;                                   // references static_var
}

int Base::compute(int x) const {
    return value_ + x;                          // uses field
}

const char* Base::label() const {
    return "Base";                              // string_literal
}

int Base::instance_count() {
    return count_;
}

// ===== Derived 实现 =====
Derived::Derived() : Base(), pt_{0, 0} {}

Derived::~Derived() {}

int Derived::compute(int x) const {
    int tmp = multiply(x, 2);                   // calls multiply
    int result = Base::compute(tmp);            // calls Base::compute (calls through overrides)
    return result + 1;
}

void Derived::tick() {
    report(LOG_INFO, "Derived::tick");          // calls report, uses LOG_INFO + string_literal
    g_verbosity++;                              // references global
}

// ===== Button 实现（多重继承）=====
void Button::draw() {
    report(LOG_DEBUG, "Button::draw");          // calls report, uses LOG_DEBUG + string_literal
}

bool Button::hit_test(int x, int y) const {
    return x > 0 && y > 0;
}

void Button::focus() {
    flush_all();                                // calls flush_all
}

// ===== Renderable / Clickable / Focusable 实现 =====
int Renderable::z_order() const { return 0; }
bool Clickable::hit_test(int x, int y) const { return false; }
void Focusable::focus() {}
void Focusable::blur() {}

// ===== Map 模板实现 =====
template <typename K, typename V>
void Map<K,V>::insert(const K& key, const V& val) {
    Node* n = new Node;
    n->key = key;
    n->value = val;
    n->next = head_;
    head_ = n;
}

template <typename K, typename V>
V Map<K,V>::find(const K& key) const {
    for (Node* n = head_; n; n = n->next) {
        if (n->key == key) return n->value;
    }
    return V();
}

template <typename K, typename V>
int Map<K,V>::size() const {
    int n = 0;
    for (Node* cur = head_; cur; cur = cur->next) n++;
    return n;
}

template <typename K, typename V>
bool Map<K,V>::empty() const {
    return head_ == nullptr;
}

template <typename K, typename V>
void Map<K,V>::clear() {
    Node* cur = head_;
    while (cur) {
        Node* next = cur->next;
        delete cur;
        cur = next;
    }
    head_ = nullptr;
}

// ===== 自由函数实现 =====
int multiply(int a, int b) {
    return a * b;
}

double divide(double a, double b) {
    if (b == 0.0) {
        report(LOG_ERROR, "division by zero");    // calls report, uses LOG_ERROR + string_literal
        return 0.0;
    }
    return a / b;
}

void report(LogLevel level, const char* msg) {
    if (g_verbosity > 0) {                        // references global
        printf("[%d] %s\n", level, msg);          // calls printf, uses string_literal via msg
    }
}

int compute_something(double scale) {
    int a = multiply(3, 4);                        // calls multiply
    int b = max_val(10, 20);                       // calls template function max_val<int> → template_use
    int c = clamp(b, 0, 100);                      // calls clamp
    return static_cast<int>((a + b + c) * scale);
}

void flush_all() {
    // uses string literal
    const char* tag = "FLUSH";                     // variable + string_literal
    report(LOG_WARN, tag);                         // calls report
}

// ===== 显式模板实例化 =====
template class Map<int, float>;
template class Map<const char*, double>;
