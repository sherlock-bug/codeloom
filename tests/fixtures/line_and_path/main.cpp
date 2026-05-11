#include "include/my_class.h"

// A standalone helper function
int helper(int a) {
    MyClass obj;
    obj.value = a;
    return obj.calc(a);
}

// Implementation of MyClass::calc
int MyClass::calc(int x) {
    return value + x;
}
