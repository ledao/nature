#include <stdio.h>
#include <stdarg.h>

// 内置函数实现
void println(const char* format, ...) {
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    printf("\n");
    va_end(args);
}

void print(const char* format, ...) {
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
}

int len(const char* str) {
    int length = 0;
    while (str[length] != '\0') {
        length++;
    }
    return length;
}

void add(int a, int b) {
}

void main() {
    println("hello nature\n");
    println("3 + 2 = ", add(3, 2));
}

int main() {
    println("hello nature\n");
    println("3 + 2 = ", add(3, 2));
    return 0;
}
