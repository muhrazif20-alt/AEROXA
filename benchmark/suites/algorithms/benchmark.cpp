#include <cstdio>
#include <cstdint>

int main() {
    std::int32_t a = 12;
    std::int32_t b = 8;
    std::int32_t c = 5;
    std::int32_t d = 3;

    std::int32_t result = (a * b) + (c * d) - (a * c) + (b * d);

    std::printf("%d\n", result);

    return 0;
}